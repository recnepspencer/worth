//! Authored static application composition.

mod action;
mod action_correspondence;
mod connection;
mod evaluated_requirement;
mod external_input;
mod feature;
mod identity;
mod instance;
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
pub use connection::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef,
    ApplicationOccurrenceConnectionBinding,
};
pub use evaluated_requirement::{
    ApplicationEvaluatedRequirement, ApplicationEvaluatedRequirementRule,
    ApplicationRequirementSubmissionDenial,
};
pub use external_input::{ApplicationExternalInputProvider, ApplicationExternalInputResolution};
pub use feature::{
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationFeatureInputDeclaration,
    ApplicationFeatureInputLeaf, ApplicationFeatureInputList, ApplicationFeatureOutputDeclaration,
    ApplicationFeatureSpec, ApplicationFeatureSpecBuilder, ApplicationInputPort,
    ApplicationOutputPort, ApplicationPortRef,
};
pub use identity::ApplicationProgramIdentity;
pub use instance::{ApplicationCompositionInstance, ApplicationRootComposition};
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
