use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
};
use worth_runtime_world::facade::{
    NoEffectCompositePublication, ProductBranchIdentity, RuntimeWorldCancellationToken,
    RuntimeWorldUnpublishedConditionalDefinition,
};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::conditional_operation::{
    QueryTemporalPredicateProvider, WorthQueryConditionalClockHandle,
};

#[derive(Debug)]
pub enum WorthQueryConditionalDefinitionPublicationDenial {
    ForeignConditionalOperation,
    ProductActivation { detail: String },
    WorldPreparation(NoEffectCompositePublication),
    BridgePreparation(worth_runtime_bridge::facade::BridgeConditionalDenial),
}

impl From<super::super::product_activation::WorthQueryProductConditionalPublicationDenial>
    for WorthQueryConditionalDefinitionPublicationDenial
{
    fn from(
        denial: super::super::product_activation::WorthQueryProductConditionalPublicationDenial,
    ) -> Self {
        use super::super::product_activation::WorthQueryProductConditionalPublicationDenial as D;
        match denial {
            D::ProductActivation(denial) => Self::ProductActivation {
                detail: format!("{denial:?}"),
            },
            D::WorldPreparation(no_effect) => Self::WorldPreparation(no_effect),
            D::BridgePreparation(denial) => Self::BridgePreparation(denial),
        }
    }
}

pub enum WorthQueryConditionalDefinitionPublicationOutcome {
    Performed(WorthQueryPerformedConditionalDefinitionPublication),
    NoEffect(NoEffectCompositePublication),
    ProductUnpublished(RuntimeWorldUnpublishedConditionalDefinition),
}

pub struct WorthQueryPerformedConditionalDefinitionPublication {
    publication: worth_runtime_world::facade::ConsumedCompositePublication,
    definition_generation: u64,
}

impl WorthQueryPerformedConditionalDefinitionPublication {
    pub fn product_branch_identity(&self) -> &ProductBranchIdentity {
        self.publication.new_product_head().branch_identity()
    }

    pub fn product_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.publication.commit().identity()
    }

    pub const fn definition_generation(&self) -> u64 {
        self.definition_generation
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    pub fn publish_conditional_definition<Node, Clock, Provider>(
        &self,
        handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        provider: Arc<Provider>,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<
        WorthQueryConditionalDefinitionPublicationOutcome,
        WorthQueryConditionalDefinitionPublicationDenial,
    >
    where
        Node: 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    {
        let operation = self
            .application()
            .conditional_operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .admit_clock(handle.binding_identity(), handle.lease())
            .ok_or(WorthQueryConditionalDefinitionPublicationDenial::ForeignConditionalOperation)?;
        let anchor = operation.lowering_anchor();
        let request = {
            let bridge_root = self.application().bridge.conditional_operations();
            let bridge = bridge_root
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let exact = bridge
                .admit_exact_conditional_signal_basis(&anchor, self.product().signal_basis())
                .map_err(|denial| {
                    WorthQueryConditionalDefinitionPublicationDenial::BridgePreparation(denial)
                })?;
            exact.installed_lowering().successor_with_wake_provider(
                QueryTemporalPredicateProvider::<Node, Provider>::new(
                    provider,
                    Arc::clone(handle.node_authority()),
                ),
            )
        };
        let outcome = self
            .application()
            .publish_product_conditional_definition(self.product(), &anchor, request, cancellation)
            .map_err(WorthQueryConditionalDefinitionPublicationDenial::from)?;
        Ok(match outcome {
            worth_runtime_world::facade::RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
                publication,
                lowering,
            } => WorthQueryConditionalDefinitionPublicationOutcome::Performed(
                WorthQueryPerformedConditionalDefinitionPublication {
                    publication: publication.consume(),
                    definition_generation: lowering.signal_definition_generation(),
                },
            ),
            worth_runtime_world::facade::RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(no_effect) => {
                WorthQueryConditionalDefinitionPublicationOutcome::NoEffect(no_effect)
            }
            worth_runtime_world::facade::RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(unpublished) => {
                WorthQueryConditionalDefinitionPublicationOutcome::ProductUnpublished(unpublished)
            }
        })
    }
}
