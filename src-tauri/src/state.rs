use crate::error::AppError;
use crate::models::CaptureJobId;

#[derive(Debug, Default)]
pub struct CaptureStateMachine {
    active: Option<CaptureJobId>,
}

impl CaptureStateMachine {
    pub fn begin(&mut self) -> Result<CaptureJobId, AppError> {
        if self.active.is_some() {
            return Err(AppError::CaptureAlreadyActive);
        }
        let id = CaptureJobId::new();
        self.active = Some(id);
        Ok(id)
    }

    pub fn ensure_active(&self, id: CaptureJobId) -> Result<(), AppError> {
        if self.active == Some(id) {
            Ok(())
        } else {
            Err(AppError::Internal)
        }
    }

    pub fn finish(&mut self, id: CaptureJobId) {
        if self.active == Some(id) {
            self.active = None;
        }
    }

    pub fn cancel(&mut self, id: CaptureJobId) -> Result<(), AppError> {
        self.ensure_active(id)?;
        self.active = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevents_overlapping_capture_jobs() {
        let mut state = CaptureStateMachine::default();
        let first = state.begin().expect("first capture");
        assert!(matches!(
            state.begin(),
            Err(AppError::CaptureAlreadyActive)
        ));
        state.finish(first);
        assert!(state.begin().is_ok());
    }

    #[test]
    fn rejects_stale_job_ids() {
        let mut state = CaptureStateMachine::default();
        let current = state.begin().expect("capture");
        state.finish(current);
        let stale = CaptureJobId::new();
        assert!(state.ensure_active(stale).is_err());
    }
}
