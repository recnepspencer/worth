use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};
use worth_runtime_world::facade::{
    NoEffectCompositePublication, PerformedCompositePublication, PlannedProductReferenceSuccessor,
    PreparedConditionalDefinitionRecovery, ProductUnpublishedRecoveryHandle,
    RuntimeWorldConditionalDefinitionPublicationOutcome, RuntimeWorldPublicationPort,
    RuntimeWorldSettledRelationalAdoptionDenial, RuntimeWorldUnpublishedConditionalDefinition,
};

use super::publication_denial::{
    WorthQueryProductPublicationDenial as Denial, WorthQueryProductPublicationOutcome as Outcome,
};
use super::request_control::WorthQueryProductPublicationRequestControl;
use super::WorthQueryProductPublicationBinding;

/// Holds the native recovery lease while Query reserves its successor delivery.
pub(crate) struct WorthQueryPreparedConditionalRecovery {
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: PreparedConditionalDefinitionRecovery,
    control: WorthQueryProductPublicationRequestControl,
}

impl WorthQueryPreparedConditionalRecovery {
    pub(crate) fn planned_successor(&self) -> &PlannedProductReferenceSuccessor {
        self.prepared.planned_successor()
    }

    pub(crate) fn unpublished_recovery_handle(&self) -> ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }

    pub(crate) fn execute(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        bridge: &BridgeSealedRuntimeAssembly,
    ) -> Result<
        Outcome<
            (
                PerformedCompositePublication,
                Arc<BridgeInstalledConditionalLowering>,
            ),
            RuntimeWorldUnpublishedConditionalDefinition,
        >,
        Denial<NoEffectCompositePublication>,
    > {
        match self
            .publication
            .execute_bridge_conditional_definition_adoption(
                execution,
                self.prepared,
                bridge,
                self.control.cancellation(),
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
    pub(crate) fn prepare_conditional_definition_adoption(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        bridge: &BridgeSealedRuntimeAssembly,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedConditionalRecovery,
        Denial<RuntimeWorldSettledRelationalAdoptionDenial>,
    > {
        let control = self.request_control(request);
        self.adoption_preparation.reached();
        let prepared = self
            .publication()
            .prepare_bridge_conditional_definition_adoption(
                handle,
                bridge,
                control.cancellation(),
                control.deadline(),
            )
            .map_err(Denial::adoption)?;
        Ok(WorthQueryPreparedConditionalRecovery {
            publication: self.publication().clone(),
            prepared,
            control,
        })
    }
}
