//! Keeps the admitted request cancellation registration alive during validation.
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Wake, Waker},
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::mvcc::{RelationalCancellationSource, RelationalOperationControl};

pub(super) struct ValidationRequestControl {
    relational: RelationalOperationControl,
    _registration: Pin<Box<dyn Future<Output = ()> + Send>>,
}

struct CancelValidation(RelationalCancellationSource);
impl Wake for CancelValidation {
    fn wake(self: Arc<Self>) {
        self.0.cancel();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.cancel();
    }
}

impl ValidationRequestControl {
    pub(super) fn new(request: &WorthQueryRequestScope) -> Self {
        let cancellation = Arc::new(CancelValidation(RelationalCancellationSource::new()));
        let waker = Waker::from(Arc::clone(&cancellation));
        let token = request.cancellation().clone();
        let mut registration: Pin<Box<dyn Future<Output = ()> + Send>> =
            Box::pin(async move { token.cancelled().await });
        if registration
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_ready()
        {
            cancellation.0.cancel();
        }
        Self {
            relational: RelationalOperationControl::from(cancellation.0.token())
                .with_deadline(request.deadline()),
            _registration: registration,
        }
    }
    pub(super) fn relational(&self) -> &RelationalOperationControl {
        &self.relational
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
    use worth_relational::facade::mvcc::RelationalOperationInterruption;

    #[test]
    fn live_request_control_preserves_cancellation_and_exact_deadline() {
        let source = WorthQueryCancellationSource::new();
        let request =
            WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(30), source.token());
        let control = ValidationRequestControl::new(&request);
        assert_eq!(control.relational().interruption(), None);
        source.cancel();
        assert_eq!(
            control.relational().interruption(),
            Some(RelationalOperationInterruption::Cancelled)
        );
        let deadline = WorthQueryRequestScope::new(
            Instant::now(),
            WorthQueryCancellationSource::new().token(),
        );
        let timed = ValidationRequestControl::new(&deadline);
        assert_eq!(
            timed.relational().interruption(),
            Some(RelationalOperationInterruption::TimedOut)
        );
    }
}
