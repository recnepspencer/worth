mod builder;
mod comparison;
mod compatibility;
mod condition;
mod condition_parameter;
mod declaration;
mod dependency;
mod host_provider_contract;
mod location;
mod markers;
mod named_clock_contract;
mod node_posture;
mod output_contract;
mod reference;
mod temporal;
mod temporal_intent_contract;
mod trigger;
mod validation;

pub use builder::WorthQueryPortableConditionalNodeBuilder;
pub use compatibility::{
    compare_portable_conditional_node_declarations, portable_conditional_node_canonical_material,
    WorthQueryPortableConditionalComparisonEquivalent,
    WorthQueryPortableConditionalComparisonMismatch,
    WorthQueryPortableConditionalComparisonOutcome,
    WorthQueryPortableConditionalComparisonUnsupported,
    WorthQueryPortableConditionalDependencyLocation, WorthQueryPortableConditionalDependencyPart,
    WorthQueryPortableConditionalDimension, WorthQueryPortableConditionalOutputPart,
};
pub use condition::{
    WorthQueryConditionalConditionClass, WorthQueryConditionalEvaluationCondition,
    WorthQueryDeltaComparisonDomain, WorthQueryDeltaThreshold,
    WorthQueryPortableConditionalConditionParts, WorthQueryPortableDeltaThresholdDenial,
    WorthQueryPortableDeltaThresholdParts, WorthQueryThresholdBoundary,
};
pub use condition_parameter::{
    WorthQueryPortableConditionParameter, WorthQueryPortableConditionParameterValue,
};
pub use declaration::{
    WorthQueryPortableConditionalNodeDeclaration, WorthQueryPortableConditionalNodeParts,
};
pub use dependency::{
    WorthQueryConditionalGraphReadRole, WorthQuerySemanticDependencyCanonicalBasis,
    WorthQuerySemanticLocality, WorthQuerySemanticTruthDependency,
    WorthQuerySemanticTruthDependencyDenial, WorthQueryTruthPartitionRole,
};
pub use host_provider_contract::{
    WorthQueryConditionalDependencyObservation, WorthQueryConditionalObservationTruthBasis,
    WorthQueryConditionalObservationView, WorthQueryConditionalObservedValue,
    WorthQueryConditionalProjectedValue, WorthQueryHostConditionalOutputComparatorProvider,
    WorthQueryHostConditionalOutputVersionProvider, WorthQueryHostConditionalPredicateProvider,
    WorthQueryHostPredicateDecision, WorthQueryHostPredicateFailure,
    WorthQueryHostPredicateFailureKind, WorthQueryHostProviderHeapRetention,
    WorthQueryHostProviderRetentionOverflow,
};
pub use location::WorthQueryConditionalNodeLocation;
pub use markers::{
    WorthQueryComparatorFamily, WorthQueryDomainConditionFamily, WorthQueryOnDemandTriggerFamily,
    WorthQueryPortableFamilyIdentityDenial, WorthQueryQuantityUnit, WorthQueryQuantityValueFamily,
    WorthQueryTypedFamilyIdentity,
};
pub use named_clock_contract::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryNamedClock, WorthQueryNamedClockFailure, WorthQueryNamedClockFailureKind,
    WorthQueryNamedClockObservation, WorthQueryNamedClockReading, WorthQueryNamedClockSource,
};
pub use node_posture::{
    WorthQueryArtifactPosture, WorthQueryConditionalNodeContext, WorthQueryConditionalNodeRole,
    WorthQueryMaintenancePosture, WorthQueryOutputRelationship,
};
pub use output_contract::{
    WorthQueryConditionalConsequenceRole, WorthQueryConditionalNodeOutput,
    WorthQueryConditionalTouchRole,
};
pub use reference::WorthQueryConditionalNodeRef;
pub use temporal::{WorthQueryTemporalCondition, WorthQueryTemporalWake};
pub use temporal_intent_contract::{
    WorthQueryTemporalIntentBounds, WorthQueryTemporalIntentCandidate,
    WorthQueryTemporalIntentIdempotencyRelation, WorthQueryTemporalIntentIdentity,
    WorthQueryTemporalIntentLifecycle, WorthQueryTemporalIntentProjectionFailure,
    WorthQueryTemporalIntentProjectionFailureKind, WorthQueryTemporalIntentProjector,
    WorthQueryTemporalIntentRevisionValue, WorthQueryTemporalOperationInputIdentity,
    MAX_TEMPORAL_DUE_WAKES_PER_OBSERVATION, MAX_TEMPORAL_INTENT_QUERY_WORK,
    MAX_TEMPORAL_INTENT_RECONSTRUCTION_ROWS,
};
pub use trigger::WorthQueryConditionalTrigger;

pub(crate) use dependency::{contract_token, dependency_token, locality_token};
pub(crate) use validation::{canonicalize_conditional_nodes, validate_conditional_nodes};

pub(crate) fn push_token(material: &mut String, label: &str, value: &str) {
    material.push_str(label);
    material.push('#');
    material.push_str(&value.len().to_string());
    material.push(':');
    material.push_str(value);
    material.push(';');
}
pub use comparison::{
    WorthQueryArtifactReuseEquivalence, WorthQueryComparatorRequirement,
    WorthQueryOutputEquivalenceRequirement,
};
