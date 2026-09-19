mod application_fact;
mod authored_declaration;
mod authored_material;
mod authored_payload_source;
mod catalog;
mod concurrency_scope;
mod confirmation_contract;
mod confirmation_route_binding;
mod consequence_contract;
mod declaration;
mod denial;
mod identity;
mod operability_contract;
mod payload_source;
mod route_binding;

pub use application_fact::{
    UiIntentApplicationFact, UiIntentApplicationFactIdentityError,
    UiIntentApplicationFactRegistrationError,
};
pub(crate) use application_fact::{
    UiIntentApplicationFactPlan, UiIntentApplicationFactSlot, UiIntentApplicationFactValue,
};
pub use authored_declaration::{
    UiIntentDeclaration, UiIntentDeclarationConcurrencyBound,
    UiIntentDeclarationConcurrencyMissing, UiIntentDeclarationConfirmationBound,
    UiIntentDeclarationConfirmationMissing, UiIntentDeclarationConsequencesBound,
    UiIntentDeclarationConsequencesMissing, UiIntentDeclarationConstructionError,
    UiIntentDeclarationOperabilityBound, UiIntentDeclarationOperabilityMissing,
};
pub(crate) use authored_material::{
    prepare_authored_intent_material, WorthUiAuthoredIntentDeclaration,
    WorthUiAuthoredIntentMaterial, WorthUiAuthoredIntentRoute,
};
pub use authored_payload_source::UiIntentPayloadSource;
pub(crate) use catalog::{
    UiIntentCatalog, UiIntentCatalogCommandRoute, UiIntentCatalogResolvedRoute,
    UiIntentCatalogSemanticComparison, UiIntentSingleProductRouteDenial,
};
pub use catalog::{UiIntentCatalogMetrics, UiIntentRouteResolutionCost};
pub use concurrency_scope::UiIntentConcurrencyScope;
pub(crate) use confirmation_contract::{
    resolve_confirmation_contract, UiResolvedIntentConfirmationContract,
    UiResolvedIntentConfirmationSource,
};
pub use confirmation_contract::{
    UiIntentConfirmationContract, UiIntentConfirmationContractIdentityError,
};
pub use confirmation_route_binding::UiIntentConfirmationRouteBinding;
pub use consequence_contract::UiIntentConsequenceContract;
pub(crate) use consequence_contract::{
    resolve_consequence_contract, UiResolvedIntentConsequenceContract,
};
pub(crate) use declaration::UiCanonicalIntentDeclaration;
pub use denial::{UiIntentCatalogPreparationDenial, UiIntentInteractionPayloadSourceKind};
pub(crate) use identity::valid_intent_identity;
pub use identity::UiIntentDeclarationIdentity;
pub(crate) use operability_contract::{
    resolve_operability_contract, UiResolvedIntentMutabilitySource,
    UiResolvedIntentOperabilityContract, UiResolvedIntentReadinessSource,
};
pub use operability_contract::{
    UiIntentMutabilitySource, UiIntentOperabilityContract,
    UiIntentOperabilityContractIdentityError, UiIntentOperabilityDependencyAxis,
    UiIntentPolicySource, UiIntentReadinessSource,
};
pub(crate) use payload_source::{
    resolve_payload_sources, UiResolvedIntentApplicationSource, UiResolvedIntentPayloadBinding,
    UiResolvedIntentPayloadSource, UiResolvedIntentProjectionSource,
};
pub use route_binding::UiIntentRouteBinding;
