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
        match publication.publish_bridge_conditional_definition(
            execution,
            prepared,
            bridge_prepared,
            bridge,
            control.cancellation(),
        ) {
            RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
                publication,
                lowering,
            } => Ok(Outcome::Performed((publication, lowering))),
            RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(effects) => {
                Ok(Outcome::ProductUnpublished(effects))
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(no_effect) => {
                Err(Denial::publication(no_effect))
            }
        }
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
        let prepared = self
            .publication()
            .prepare_with_signal(
                self.observation().clone(),
                intent,
                control.cancellation(),
                control.deadline(),
            )
            .map_err(Denial::publication)?;
        Ok(WorthQueryPreparedCombinedProductPublication {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
