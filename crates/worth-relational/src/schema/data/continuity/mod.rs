mod boundary_fingerprint;
mod descriptor_versions;
mod diff_atoms;
mod transition_classification;
mod transition_plans;

pub use boundary_fingerprint::SchemaBoundaryFingerprint;
pub use descriptor_versions::{
    runtime_descriptor_canonical_basis_policy, runtime_descriptor_semantics_policy,
    DescriptorCanonicalBasisSupportPolicy, DescriptorCanonicalBasisVersion,
    DescriptorSemanticsSupportPolicy, DescriptorSemanticsVersion,
};
pub use diff_atoms::{
    SchemaChangeText, SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind, SchemaElementRef,
};
pub use transition_classification::{
    default_boundary_visibility_for_continuation,
    default_boundary_visibility_for_subscriber_impact, FreeFormSchemaDiffIntent,
    HistoricalInterpretationSensitivity, SchemaBridgeabilityClassification,
    SchemaContinuationAdmissionObservation, SchemaContinuationClassification,
    SchemaLineageOrderingSemantics, SchemaPublicationImpact, SchemaReconciliationClassification,
    SchemaReconciliationOrderingMode, SchemaReconciliationPolicy, SchemaStratum,
    SchemaSubscriberImpact, SchemaTransitionBarrier, SubscriberBoundaryVisibility,
};
pub use transition_plans::{
    LoweredSchemaTransitionPlan, ProposedSchemaTransition, SchemaBridgeDescriptor,
    SchemaContinuationDescriptor, SchemaLineageArtifact, SchemaReconciliationDescriptor,
    SchemaTransitionArtifact, SchemaTransitionSummary, ValidatedSchemaTransition,
};

pub(crate) use diff_atoms::{SchemaTextReadmissionMemory, SchemaTransitionReadmissionMemory};
