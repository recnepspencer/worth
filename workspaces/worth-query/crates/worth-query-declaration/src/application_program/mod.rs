//! Authored static application composition.

mod connection;
mod feature;
mod identity;
mod instance;
mod output_graph;
mod program;
mod rule;

pub use connection::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef,
    ApplicationOccurrenceConnectionBinding,
};
pub use feature::{
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationFeatureInputDeclaration,
    ApplicationFeatureInputLeaf, ApplicationFeatureInputList, ApplicationFeatureInstanceRef,
    ApplicationFeatureLeaf, ApplicationFeatureList, ApplicationFeatureRef, ApplicationFeatureShape,
    ApplicationInputPort, ApplicationOutputPort, ApplicationPortRef,
    ApplicationProgramFeaturesShape,
};
pub use identity::ApplicationProgramIdentity;
pub use instance::{ApplicationCompositionInstance, ApplicationRootComposition};
pub use output_graph::{
    ApplicationConnectionShape, ApplicationOutputChildrenShape, ApplicationOutputEdge,
    ApplicationOutputEdgeShape, ApplicationOutputEdgesShape, ApplicationOutputGraph,
    ApplicationOutputGraphShape, ApplicationOutputLeaf, ApplicationProgramRootConnection,
    ApplicationProgramRootConnectionRef, ApplicationProgramRootEdges,
};
pub use program::{
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramValidationDenial,
    ApplicationProgramValidationDenialKind, ValidatedApplicationProgram,
};
pub use rule::{
    ApplicationCommitBoundary, ApplicationLocalRuleInstanceRef, ApplicationLocalRuleRef,
    ApplicationMutationSensitive, ApplicationProgramRuleDeclaration, ApplicationProgramRulesShape,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ApplicationSharedRuleInstanceRef,
    ApplicationSharedRuleRef, ApplicationSnapshotPublication,
};
