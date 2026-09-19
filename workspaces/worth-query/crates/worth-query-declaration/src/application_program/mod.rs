//! Authored static application composition.

mod action;
mod action_correspondence;
mod change_shape;
mod connection;
mod derived_artifact;
mod derived_collection;
mod evaluated_requirement;
mod evolution;
mod external_input;
mod feature;
mod identity;
mod instance;
mod locality;
mod managed_computation;
mod manifest;
mod output_graph;
mod output_posture;
mod program;
mod program_outputs;
mod rule;

#[cfg(test)]
mod program_tests;

pub use action::{ApplicationActionCorrespondenceDeclaration, ApplicationActionDeclaration};
pub use action_correspondence::{
    ApplicationOptionalMemberEdit, ApplicationRepeatedOptionalMemberCorrespondence,
};
pub use change_shape::{
    ApplicationChangePosture, ApplicationChangeShape, ApplicationChangeShapeDeclaration,
};
pub use connection::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef,
    ApplicationOccurrenceConnectionBinding,
};
pub use derived_artifact::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
    ApplicationDerivedArtifactDeclaration, ApplicationDerivedArtifactGovernance,
};
pub use derived_collection::{
    ApplicationCollectionContributor, ApplicationCollectionGrouping,
    ApplicationCollectionIncompletePosture, ApplicationCollectionLineage,
    ApplicationCollectionMeasures, ApplicationDerivedCollection,
    ApplicationDerivedCollectionDeclaration,
};
pub use evaluated_requirement::{
    ApplicationEvaluatedRequirement, ApplicationEvaluatedRequirementRule,
    ApplicationRequirementSubmissionDenial,
};
pub use evolution::ApplicationProgramRevision;
pub(in crate::application_program) use evolution::ApplicationProgramRevisionBudgetDenial;
pub use external_input::{ApplicationExternalInputProvider, ApplicationExternalInputResolution};
pub use feature::{
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationFeatureInputDeclaration,
    ApplicationFeatureInputLeaf, ApplicationFeatureInputList, ApplicationFeatureOutputDeclaration,
    ApplicationFeatureSpec, ApplicationFeatureSpecBuilder, ApplicationInputPort,
    ApplicationOutputPort, ApplicationPortRef,
};
pub use identity::ApplicationProgramIdentity;
pub use instance::{ApplicationCompositionInstance, ApplicationRootComposition};
pub use locality::{
    ApplicationLocalityDeclaration, ApplicationLocalityGranule, ApplicationLocalityScope,
};
pub use managed_computation::{
    ApplicationComputationExecution, ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling, ApplicationComputationReuse,
    ApplicationComputationStopped, ApplicationManagedComputation,
    ApplicationManagedComputationDeclaration,
};
pub use manifest::ApplicationProgramManifest;
pub use output_graph::{
    ApplicationConnectionShape, ApplicationDiscoveredOutputGraph, ApplicationDiscoveredOutputRoot,
    ApplicationOutputChildrenShape, ApplicationOutputEdge, ApplicationOutputEdgeShape,
    ApplicationOutputEdgesShape, ApplicationOutputGraph, ApplicationOutputGraphShape,
    ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
};
pub use output_posture::{ApplicationNoOutputGraph, ApplicationProgramOutputShape};
pub use program::{
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramValidationDenial,
    ApplicationProgramValidationDenialKind, ValidatedApplicationProgram,
};
pub use program_outputs::{
    ApplicationProgramOutputRootsShape, ApplicationProgramOutputs, ApplicationProgramOutputsShape,
};
pub use rule::{
    ApplicationCommitBoundary, ApplicationLocalRuleInstanceRef, ApplicationLocalRuleRef,
    ApplicationMutationSensitive, ApplicationProgramRuleDeclaration, ApplicationProgramRulesShape,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ApplicationSharedRuleInstanceRef,
    ApplicationSharedRuleRef, ApplicationSnapshotPublication,
};
