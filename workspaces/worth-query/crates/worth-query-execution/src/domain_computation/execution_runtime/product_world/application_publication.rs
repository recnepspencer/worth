use super::publication_denial::{
    WorthQueryProductPublicationDenial as Denial, WorthQueryProductPublicationOutcome as Outcome,
};
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_bridge::facade::{
    BridgePreparedConditionalInstallationExtension, BridgeSealedRuntimeAssembly,
};
use worth_runtime_world::facade::{
    CompositePublicationIntent, NoEffectCompositePublication,
    PreparedCompositePublicationWithSignal, RuntimeWorldPublicationPort,
};

use super::{
    request_control::WorthQueryProductPublicationRequestControl,
    WorthQueryProductPublicationBinding,
};

#[must_use = "a prepared combined publication owns a reserved World attempt"]
pub(crate) struct WorthQueryPreparedCombinedProductPublication {
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: PreparedCompositePublicationWithSignal,
    control: WorthQueryProductPublicationRequestControl,
}

impl WorthQueryPreparedCombinedProductPublication {
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
        bridge_prepared: BridgePreparedConditionalInstallationExtension,
        bridge: &BridgeSealedRuntimeAssembly,
    ) -> Result<
        Outcome<
            (
                worth_runtime_world::facade::PerformedCompositePublication,
                Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
            ),
            worth_runtime_world::facade::RuntimeWorldUnpublishedConditionalDefinition,
        >,
        Denial<NoEffectCompositePublication>,
    > {
        let Self {
            publication,
            prepared,
            control,
        } = self;
        super::request_control::publish_conditional_definition(
            &publication,
            execution,
            prepared,
            bridge_prepared,
            bridge,
            control.cancellation(),
        )
    }
}

impl WorthQueryProductPublicationBinding {
    pub(crate) fn prepare_combined_candidate(
        &self,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
    ) -> Result<WorthQueryPreparedCombinedProductPublication, Denial<NoEffectCompositePublication>>
    {
        let control = self.request_control(request);
        let intent = CompositePublicationIntent::with_signal(Some(
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        ))
        .with_prepared_relational_candidate(candidate);
        let intent = if successor_observation_requested {
            intent.with_successor_observation()
        } else {
            intent
        };
        let prepared = super::request_control::prepare_conditional_publication(
            self.publication(),
            self.observation().clone(),
            intent,
            control.cancellation(),
            control.deadline(),
        )?;
        Ok(WorthQueryPreparedCombinedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
