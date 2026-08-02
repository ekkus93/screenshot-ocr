use crate::cancellation::CancellationToken;
use crate::error::AppError;
use crate::models::CaptureJobId;

#[derive(Debug)]
struct ActiveCapture {
    id: CaptureJobId,
    cancellation: CancellationToken,
    started: bool,
}

#[derive(Debug, Default)]
pub struct CaptureStateMachine {
    active: Option<ActiveCapture>,
}

impl CaptureStateMachine {
    pub fn reserve(&mut self, id: CaptureJobId) -> Result<(), AppError> {
        if self.active.is_some() {
            return Err(AppError::CaptureAlreadyActive);
        }
        self.active = Some(ActiveCapture {
            id,
            cancellation: CancellationToken::new(),
            started: false,
        });
        Ok(())
    }

    pub fn begin(&mut self, id: CaptureJobId) -> Result<CancellationToken, AppError> {
        match &mut self.active {
            None => {
                let cancellation = CancellationToken::new();
                self.active = Some(ActiveCapture {
                    id,
                    cancellation: cancellation.clone(),
                    started: true,
                });
                Ok(cancellation)
            }
            Some(active) if active.id == id && !active.started => {
                active.started = true;
                Ok(active.cancellation.clone())
            }
            Some(_) => Err(AppError::CaptureAlreadyActive),
        }
    }

    pub fn active_job_id(&self) -> Option<CaptureJobId> {
        self.active.as_ref().map(|active| active.id)
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

    pub fn cancel_active(&self) -> bool {
        if let Some(active) = &self.active {
            active.cancellation.cancel();
            true
        } else {
            false
        }
    }

    pub fn expire_reservation(&mut self, id: CaptureJobId) -> bool {
        let should_expire = self
            .active
            .as_ref()
            .is_some_and(|active| active.id == id && !active.started);
        if should_expire {
            self.active = None;
        }
        should_expire
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
    fn reserved_job_is_consumed_by_the_matching_capture() {
        let mut state = CaptureStateMachine::default();
        let job_id = CaptureJobId::new();
        state.reserve(job_id).expect("reserve");
        assert_eq!(state.active_job_id(), Some(job_id));
        state.begin(job_id).expect("begin reserved job");
        assert!(matches!(
            state.begin(job_id),
            Err(AppError::CaptureAlreadyActive)
        ));
    }

    #[test]
    fn reservation_expiration_never_releases_a_started_job() {
        let mut state = CaptureStateMachine::default();
        let job_id = CaptureJobId::new();
        state.reserve(job_id).expect("reserve");
        assert!(state.expire_reservation(job_id));

        let started = CaptureJobId::new();
        state.begin(started).expect("begin");
        assert!(!state.expire_reservation(started));
        assert_eq!(state.active_job_id(), Some(started));
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
    fn cancel_active_is_idempotent_and_preserves_ownership() {
        let mut state = CaptureStateMachine::default();
        let current = CaptureJobId::new();
        state.begin(current).expect("capture");
        assert!(state.cancel_active());
        assert!(state.cancel_active());
        assert_eq!(state.active_job_id(), Some(current));
        state.finish(current);
        assert!(!state.cancel_active());
    }

    #[test]
    fn rejects_stale_job_ids() {
        let mut state = CaptureStateMachine::default();
        let current = CaptureJobId::new();
        state.begin(current).expect("capture");
        state.finish(current);
        let stale = CaptureJobId::new();
        assert!(state.ensure_not_cancelled(stale).is_err());
        assert!(state.cancel(stale).is_err());
    }
}
