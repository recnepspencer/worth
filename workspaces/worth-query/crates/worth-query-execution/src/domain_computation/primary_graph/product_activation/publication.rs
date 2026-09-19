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
    ProductActivation(WorthQueryProductActivationDenial),
    WorldPreparation(NoEffectCompositePublication),
    BridgePreparation(BridgeConditionalDenial),
}

impl From<WorthQueryProductActivationDenial> for WorthQueryProductConditionalPublicationDenial {
    fn from(denial: WorthQueryProductActivationDenial) -> Self {
        Self::ProductActivation(denial)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn publish_product_conditional_definition(
        &self,
        product: &WorthQueryProductBranchLease,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<
        RuntimeWorldConditionalDefinitionPublicationOutcome,
        WorthQueryProductConditionalPublicationDenial,
    > {
        let identity = product.branch_identity();
        let gate = self.product_runtime.activations.gate(identity)?;
        gate.publish(|| {
            let prepared_world = match self
                .product_runtime
                .owner
                .publication_port()
                .prepare_with_signal(
                    product.observation().clone(),
                    CompositePublicationIntent::with_signal(None),
                    cancellation,
                    None,
                ) {
                Ok(prepared) => prepared,
                Err(denial) => {
                    return Err(
                        WorthQueryProductConditionalPublicationDenial::WorldPreparation(denial),
                    )
                }
            };
            let bridge = self.bridge.conditional();
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
            let outcome = self
                .product_runtime
                .owner
                .publication_port()
                .publish_bridge_conditional_definition(
                    prepared_world,
                    prepared_bridge,
                    &bridge,
                    cancellation,
                );
            Ok(outcome)
        })?
    }
}
