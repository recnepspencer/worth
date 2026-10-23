use std::sync::Arc;

mod admission;
mod application_change;
pub use application_change::WorthQueryAdmittedApplicationConditionalDefinition;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
};
use worth_runtime_world::facade::{
    NoEffectCompositePublication, ProductBranchIdentity, RuntimeWorldCancellationToken,
    RuntimeWorldUnpublishedConditionalDefinition,
};

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalClockHandle;

/// Why a conditional definition was not admitted for an application change.
/// Admission is a check, so nothing was published.
#[derive(Debug)]
pub enum WorthQueryApplicationConditionalDefinitionAdmissionDenial {
    /// The operation has no admitted mutation product to bind a successor to.
    MissingProductObservation,
    /// The installed registry has no matching typed conditional binding.
    ConditionalBindingUnavailable { identity: &'static str },
    /// The clock handle's conditional operation is not live in this
    /// application.
    ForeignConditionalOperation,
    /// The bridge refused the conditional signal basis of the selected product.
    BridgePreparation(worth_runtime_bridge::facade::BridgeConditionalDenial),
}

/// Why publishing a conditional definition was refused before publication.
/// Nothing was published.
#[derive(Debug)]
pub enum WorthQueryConditionalDefinitionPublicationDenial {
    /// The host call was refused admission to its execution request.
    ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The clock handle's conditional operation is not live in this
    /// application.
    ForeignConditionalOperation,
    /// The product's active program refused the definition; the detail says
    /// why.
    ProductActivation { detail: String },
    /// The product world refused to prepare the publication.
    WorldPreparation(NoEffectCompositePublication),
    /// The bridge refused the conditional signal basis of the selected product.
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
            D::ExecutionRequest(cause) => Self::ExecutionRequest(cause),
            D::ProductActivation(denial) => Self::ProductActivation {
                detail: format!("{denial:?}"),
            },
            D::WorldPreparation(no_effect) => Self::WorldPreparation(no_effect),
            D::BridgePreparation(denial) => Self::BridgePreparation(denial),
        }
    }
}

/// How a conditional definition publication that passed its checks ended.
pub enum WorthQueryConditionalDefinitionPublicationOutcome {
    /// The definition was published and the product head moved.
    Performed(WorthQueryPerformedConditionalDefinitionPublication),
    /// Nothing was published; the publication says why.
    NoEffect(NoEffectCompositePublication),
    /// Some owners moved, but the product head did not. Recover publication.
    ProductUnpublished(RuntimeWorldUnpublishedConditionalDefinition),
}

/// A performed conditional definition publication: the product branch and
/// commit it landed on, and the signal definition generation it installed.
pub struct WorthQueryPerformedConditionalDefinitionPublication {
    publication: worth_runtime_world::facade::ConsumedCompositePublication,
    definition_generation: u64,
}

impl WorthQueryPerformedConditionalDefinitionPublication {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(
            self.publication.new_product_head().lifecycle_incarnation(),
        )
    }

    #[doc(hidden)]
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
    pub fn admit_application_conditional_definition<Node, Clock, Provider>(
        &self,
        handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        provider: Arc<Provider>,
    ) -> Result<
        WorthQueryAdmittedApplicationConditionalDefinition,
        WorthQueryApplicationConditionalDefinitionAdmissionDenial,
    >
    where
        Node: 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    {
        self.application()
            .admit_conditional_definition_on_product(self.product(), handle, provider)
    }

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
        self.application().with_host_advancement(|active_phase| {
            let phase = &active_phase;

        let (anchor, request) = admission::definition_request(self.application(), self.product(), handle, provider)?;
        let outcome = self
            .application()
            .publish_product_conditional_definition(phase, self.product(), &anchor, request, cancellation)
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

        }).map_err(WorthQueryConditionalDefinitionPublicationDenial::ExecutionRequest)?
    }
}
