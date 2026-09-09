use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Wake, Waker};

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_world::facade::{
    CompositePublicationIntent, NoEffectCompositePublication,
    PreparedCompositePublicationWithoutSignal, RuntimeWorldCancellationSource,
    RuntimeWorldCancellationToken, RuntimeWorldPublicationOutcome, RuntimeWorldPublicationPort,
};

use super::WorthQueryProductPublicationBinding;

#[must_use = "a prepared product publication owns a reserved World attempt"]
pub(crate) struct WorthQueryPreparedProductPublication {
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: PreparedCompositePublicationWithoutSignal,
    control: WorthQueryProductPublicationRequestControl,
}

impl WorthQueryPreparedProductPublication {
    pub(crate) fn unpublished_recovery_handle(
        &self,
    ) -> worth_runtime_world::facade::ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }

    pub(crate) fn execute(self) -> RuntimeWorldPublicationOutcome {
        let Self {
            publication,
            prepared,
            control,
        } = self;
        publication.execute_without_signal(prepared, control.cancellation())
    }
}

pub(crate) struct WorthQueryProductPublicationRequestControl {
    cancellation: RuntimeWorldCancellationToken,
    deadline: worth_runtime_world::facade::RuntimeWorldInstant,
    _request_cancellation: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl WorthQueryProductPublicationRequestControl {
    pub(crate) fn cancellation(&self) -> &RuntimeWorldCancellationToken {
        &self.cancellation
    }

    pub(crate) const fn deadline(&self) -> worth_runtime_world::facade::RuntimeWorldInstant {
        self.deadline
    }
}

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
    pub(crate) fn request_control(
        &self,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryProductPublicationRequestControl {
        let cancellation = Arc::new(PublicationCancellation(
            RuntimeWorldCancellationSource::new(),
        ));
        let waker = Waker::from(Arc::clone(&cancellation));
        let request_cancellation = request.cancellation().clone();
        let mut request_cancellation: Pin<Box<dyn Future<Output = ()> + Send>> =
            Box::pin(async move { request_cancellation.cancelled().await });
        if request_cancellation
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_ready()
        {
            cancellation.0.cancel();
        }
        WorthQueryProductPublicationRequestControl {
            cancellation: cancellation.0.token(),
            deadline: self.deadline(request.deadline()),
            _request_cancellation: request_cancellation,
        }
    }

    pub(crate) fn prepare_relational_candidate(
        &self,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedProductPublication, NoEffectCompositePublication> {
        let control = self.request_control(request);
        let intent = CompositePublicationIntent::without_signal(
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .with_prepared_relational_candidate(candidate);
        let prepared = self.publication().prepare_without_signal(
            self.observation().clone(),
            intent,
            control.cancellation(),
            Some(control.deadline()),
        )?;
        Ok(WorthQueryPreparedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
