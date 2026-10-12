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

use super::publication_denial::{
    WorthQueryProductPublicationDenial as Denial, WorthQueryProductPublicationOutcome as Outcome,
};
use super::WorthQueryProductPublicationBinding;

#[must_use = "a prepared product publication owns a reserved World attempt"]
pub(crate) struct WorthQueryPreparedProductPublication {
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: PreparedCompositePublicationWithoutSignal,
    control: WorthQueryProductPublicationRequestControl,
}

impl WorthQueryPreparedProductPublication {
    pub(crate) fn planned_successor(
        &self,
    ) -> &worth_runtime_world::facade::PlannedProductReferenceSuccessor {
        self.prepared.planned_successor()
    }

    pub(crate) fn unpublished_recovery_handle(
        &self,
    ) -> worth_runtime_world::facade::ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }

    pub(crate) fn execute(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Outcome, Denial<NoEffectCompositePublication>> {
        let Self {
            publication,
            prepared,
            control,
        } = self;
        match publication.execute_without_signal(execution, prepared, control.cancellation()) {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                Ok(Outcome::Performed(performed))
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                Ok(Outcome::ProductUnpublished(effects))
            }
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
                Err(Denial::publication(no_effect))
            }
        }
    }
}

pub(crate) struct WorthQueryProductPublicationRequestControl {
    cancellation: RuntimeWorldCancellationToken,
    deadline: Option<worth_runtime_world::facade::RuntimeWorldInstant>,
    _request_cancellation: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl WorthQueryProductPublicationRequestControl {
    pub(crate) fn cancellation(&self) -> &RuntimeWorldCancellationToken {
        &self.cancellation
    }

    pub(crate) const fn deadline(
        &self,
    ) -> Option<worth_runtime_world::facade::RuntimeWorldInstant> {
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
    pub(crate) fn prepare_settled_relational_adoption(
        &self,
        unpublished: &crate::domain_computation::WorthQueryProductUnpublishedApplication,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        Denial<worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial>,
    > {
        let control = self.request_control(request);
        self.adoption_preparation.reached();
        let prepared = unpublished
            .prepare_settled_relational_adoption(control.cancellation(), control.deadline())
            .map_err(Denial::adoption)?;
        Ok(WorthQueryPreparedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }

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
            deadline: request.deadline().map(|deadline| self.deadline(deadline)),
            _request_cancellation: request_cancellation,
        }
    }

    pub(crate) fn prepare_relational_candidate(
        &self,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
    ) -> Result<WorthQueryPreparedProductPublication, Denial<NoEffectCompositePublication>> {
        let control = self.request_control(request);
        let intent = CompositePublicationIntent::without_signal(
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .with_prepared_relational_candidate(candidate);
        let intent = if successor_observation_requested {
            intent.with_successor_observation()
        } else {
            intent
        };
        let prepared = self
            .publication()
            .prepare_without_signal(
                self.observation().clone(),
                intent,
                control.cancellation(),
                control.deadline(),
            )
            .map_err(Denial::publication)?;
        Ok(WorthQueryPreparedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
