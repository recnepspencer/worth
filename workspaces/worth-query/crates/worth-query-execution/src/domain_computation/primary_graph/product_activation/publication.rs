use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::execution_runtime::product_world::{
    prepare_conditional_publication, publish_conditional_definition,
};
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeInstalledConditionalLowering,
    BridgeOwnedConditionalInstallationRequest,
};
use worth_runtime_world::facade::{
    CompositePublicationIntent, NoEffectCompositePublication, RuntimeWorldCancellationToken,
    RuntimeWorldConditionalDefinitionPublicationOutcome,
};

use crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductBranchLease,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProductConditionalPublicationDenial
{
    ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    ProductActivation(WorthQueryProductActivationDenial),
    WorldPreparation(NoEffectCompositePublication),
    BridgePreparation(BridgeConditionalDenial),
    AuthoredBridgePreparation(
        worth_runtime_bridge::facade::BridgeDurableDefinitionPublicationDenial,
    ),
    DurableOpen(worth_runtime_world::facade::WorldOpenAttemptRefusal),
    DurablePreparation(worth_runtime_world::facade::WorldNativePreparationRefusal),
}

impl From<WorthQueryProductActivationDenial> for WorthQueryProductConditionalPublicationDenial {
    fn from(denial: WorthQueryProductActivationDenial) -> Self {
        Self::ProductActivation(denial)
    }
}

mod authored;
impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn publish_product_conditional_definition(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        product: &WorthQueryProductBranchLease,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<
        RuntimeWorldConditionalDefinitionPublicationOutcome,
        WorthQueryProductConditionalPublicationDenial,
    > {
        let identity = product.branch_identity();
        let gate = self.runtime.product_runtime.activations.gate(identity)?;
        gate.publish(|| {
            let owner=self.runtime.bridge.conditional();
            if let crate::domain_computation::primary_graph::managed_bridge::conditional_execution::ConditionalExecutionOwner::Authored(bridge)=&owner {
                return authored::publish(self,phase,product,bridge,predecessor,request,cancellation);
            }
            let publication = self.runtime.product_runtime.owner.publication_port();
            let prepared_world = match prepare_conditional_publication(&publication, product.observation().clone(), CompositePublicationIntent::with_signal(None), cancellation, None) {
                Ok(prepared) => prepared,
                Err(PublicationDenial::ClockMismatch) => return Err(WorthQueryProductConditionalPublicationDenial::ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock))),
Err(PublicationDenial::Stopped(stop)) => return Err(WorthQueryProductConditionalPublicationDenial::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(stop))),
                Err(PublicationDenial::Refused(denial)) => return Err(WorthQueryProductConditionalPublicationDenial::WorldPreparation(denial)),
            };
            let bridge_owner = self.runtime.bridge.conditional();
            let bridge = bridge_owner.memory().map_err(WorthQueryProductConditionalPublicationDenial::BridgePreparation)?;
            let predecessor = bridge
                .admit_exact_conditional_signal_basis(
                    predecessor,
                    product.observation().basis().signal_basis(),
                )
                .map_err(WorthQueryProductConditionalPublicationDenial::BridgePreparation)?;
            let prepared_bridge = match bridge
                .prepare_owned_conditional_definition_successor(&predecessor, request)
            {
                Ok(prepared) => prepared,
                Err(denial) => {
                    return Err(
                        WorthQueryProductConditionalPublicationDenial::BridgePreparation(denial),
                    )
                }
            };
            match publish_conditional_definition(&publication, phase.execution_request_for(&self.runtime.product_runtime).map_err(|cause| WorthQueryProductConditionalPublicationDenial::ExecutionRequest(cause.into()))?, prepared_world, prepared_bridge, &bridge, cancellation) {
                Ok(PublicationOutcome::Performed((publication, lowering))) => Ok(RuntimeWorldConditionalDefinitionPublicationOutcome::Performed { publication, lowering }),
                Ok(PublicationOutcome::ProductUnpublished(effects)) => Ok(RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(effects)),
                Err(PublicationDenial::ClockMismatch) => Err(WorthQueryProductConditionalPublicationDenial::ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock))),
Err(PublicationDenial::Stopped(stop)) => Err(WorthQueryProductConditionalPublicationDenial::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(stop))),
                Err(PublicationDenial::Refused(no_effect)) => Ok(RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(no_effect)),
            }
        })?
    }
}
