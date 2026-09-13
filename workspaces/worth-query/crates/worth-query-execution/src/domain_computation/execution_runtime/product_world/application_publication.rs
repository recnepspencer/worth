use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_bridge::facade::{
    BridgePreparedConditionalInstallationExtension, BridgeSealedRuntimeAssembly,
};
use worth_runtime_world::facade::{
    CompositePublicationIntent, NoEffectCompositePublication,
    PreparedCompositePublicationWithSignal, RuntimeWorldConditionalDefinitionPublicationOutcome,
    RuntimeWorldPublicationPort,
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
    pub(crate) fn unpublished_recovery_handle(
        &self,
    ) -> worth_runtime_world::facade::ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }

    pub(crate) fn execute(
        self,
        bridge_prepared: BridgePreparedConditionalInstallationExtension,
        bridge: &BridgeSealedRuntimeAssembly,
    ) -> RuntimeWorldConditionalDefinitionPublicationOutcome {
        let Self {
            publication,
            prepared,
            control,
        } = self;
        publication.publish_bridge_conditional_definition(
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
    ) -> Result<WorthQueryPreparedCombinedProductPublication, NoEffectCompositePublication> {
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
        let prepared = self.publication().prepare_with_signal(
            self.observation().clone(),
            intent,
            control.cancellation(),
            Some(control.deadline()),
        )?;
        Ok(WorthQueryPreparedCombinedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
