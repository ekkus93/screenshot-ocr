use crate::cancellation::CancellationToken;
use crate::error::AppError;
use crate::models::CaptureJobId;

#[derive(Debug)]
struct ActiveCapture {
    id: CaptureJobId,
    cancellation: CancellationToken,
}

#[derive(Debug, Default)]
pub struct CaptureStateMachine {
    active: Option<ActiveCapture>,
}

impl CaptureStateMachine {
    pub fn begin(&mut self, id: CaptureJobId) -> Result<CancellationToken, AppError> {
        if self.active.is_some() {
            return Err(AppError::CaptureAlreadyActive);
        }
        let cancellation = CancellationToken::new();
        self.active = Some(ActiveCapture {
            id,
            cancellation: cancellation.clone(),
        });
        Ok(cancellation)
    }

    pub fn ensure_active(&self, id: CaptureJobId) -> Result<(), AppError> {
        match &self.active {
            Some(active) if active.id == id => Ok(()),
            _ => Err(AppError::Internal),
        }
    }

    pub fn ensure_not_cancelled(&self, id: CaptureJobId) -> Result<(), AppError> {
        let active = self.active_for(id)?;
        active.cancellation.check()
    }

    pub fn finish(&mut self, id: CaptureJobId) {
        if self.active.as_ref().is_some_and(|active| active.id == id) {
            self.active = None;
        }
    }

    pub fn cancel(&self, id: CaptureJobId) -> Result<(), AppError> {
        let active = self.active_for(id)?;
        active.cancellation.cancel();
        Ok(())
    }

    fn active_for(&self, id: CaptureJobId) -> Result<&ActiveCapture, AppError> {
        match &self.active {
            Some(active) if active.id == id => Ok(active),
            _ => Err(AppError::Internal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevents_overlapping_capture_jobs() {
        let mut state = CaptureStateMachine::default();
        let first = CaptureJobId::new();
        state.begin(first).expect("first capture");
        assert!(matches!(
            state.begin(CaptureJobId::new()),
            Err(AppError::CaptureAlreadyActive)
        ));
        state.finish(first);
        assert!(state.begin(CaptureJobId::new()).is_ok());
    }

    #[test]
    fn cancellation_does_not_release_job_ownership() {
        let mut state = CaptureStateMachine::default();
        let current = CaptureJobId::new();
        state.begin(current).expect("capture");
        state.cancel(current).expect("cancel capture");
        assert!(matches!(
            state.begin(CaptureJobId::new()),
            Err(AppError::CaptureAlreadyActive)
        ));
        assert!(matches!(
            state.ensure_not_cancelled(current),
            Err(AppError::CaptureCancelled)
        ));
        state.finish(current);
        assert!(state.begin(CaptureJobId::new()).is_ok());
    }

    #[test]
    fn rejects_stale_job_ids() {
        let mut state = CaptureStateMachine::default();
        let current = CaptureJobId::new();
        state.begin(current).expect("capture");
        state.finish(current);
        let stale = CaptureJobId::new();
        assert!(state.ensure_active(stale).is_err());
        assert!(state.cancel(stale).is_err());
    }
}
