use crate::error::AppError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone, Debug)]
pub struct CancellationToken {
    inner: Arc<CancellationState>,
}

#[derive(Debug)]
struct CancellationState {
    cancelled: AtomicBool,
    notify: Notify,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancellationState {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub fn cancel(&self) -> bool {
        if self.inner.cancelled.swap(true, Ordering::AcqRel) {
            false
        } else {
            self.inner.notify.notify_waiters();
            true
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Result<(), AppError> {
        if self.is_cancelled() {
            Err(AppError::CaptureCancelled)
        } else {
            Ok(())
        }
    }

    pub async fn cancelled(&self) {
        loop {
            let notified = self.inner.notify.notified();
            if self.is_cancelled() {
                return;
            }
            notified.await;
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[test]
    fn cancellation_is_idempotent() {
        let token = CancellationToken::new();
        assert!(token.cancel());
        assert!(!token.cancel());
        assert!(matches!(token.check(), Err(AppError::CaptureCancelled)));
    }

    #[tokio::test]
    async fn cancellation_wakes_waiters() {
        let token = CancellationToken::new();
        let waiter = token.clone();
        let task = tokio::spawn(async move {
            waiter.cancelled().await;
        });
        token.cancel();
        timeout(Duration::from_secs(1), task)
            .await
            .expect("cancellation waiter timed out")
            .expect("cancellation waiter failed");
    }

    #[tokio::test]
    async fn pre_cancelled_token_does_not_miss_notification() {
        let token = CancellationToken::new();
        token.cancel();
        timeout(Duration::from_secs(1), token.cancelled())
            .await
            .expect("pre-cancelled waiter timed out");
    }
}
