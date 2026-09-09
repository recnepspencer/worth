use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Wake, Waker};

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_world::facade::{
    CompositePublicationIntent, RuntimeWorldCancellationSource, RuntimeWorldPublicationOutcome,
};

use super::WorthQueryProductPublicationBinding;

struct PublicationCancellation(RuntimeWorldCancellationSource);

impl Wake for PublicationCancellation {
    fn wake(self: Arc<Self>) {
        self.0.cancel();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.cancel();
    }
}

impl WorthQueryProductPublicationBinding {
    pub(crate) fn publish_relational_candidate(
        &self,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
    ) -> RuntimeWorldPublicationOutcome {
        let cancellation = Arc::new(PublicationCancellation(
            RuntimeWorldCancellationSource::new(),
        ));
        let waker = Waker::from(Arc::clone(&cancellation));
        let mut pending = std::pin::pin!(request.cancellation().cancelled());
        if pending
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_ready()
        {
            cancellation.0.cancel();
        }
        let intent = CompositePublicationIntent::without_signal(
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .with_prepared_relational_candidate(candidate);
        let token = cancellation.0.token();
        match self.publication().prepare_without_signal(
            self.observation().clone(),
            intent,
            &token,
            Some(self.deadline(request.deadline())),
        ) {
            Ok(prepared) => self.publication().execute_without_signal(prepared, &token),
            Err(no_effect) => RuntimeWorldPublicationOutcome::NoEffect(no_effect),
        }
    }
}
