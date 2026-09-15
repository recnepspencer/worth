//! Authored static application composition.

mod binding;
mod connection;
mod feature;
mod identity;
mod inventory;
mod program;
mod rule;
mod validation;
#[cfg(test)]
mod validation_tests;

pub use binding::{
    ApplicationProgramConnectionNode, ApplicationProgramConnectionRole,
    ApplicationProgramConnectionSet, ApplicationProgramDependentConnection,
    ApplicationProgramRequiredConnection, ApplicationProgramUnavailableConnection,
};

pub use connection::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity, ApplicationConnectionRef,
    ApplicationOccurrenceConnectionBinding,
};
pub use feature::{
    ApplicationFeature, ApplicationFeatureAvailable, ApplicationFeatureDeclaration,
    ApplicationFeaturePosture, ApplicationFeaturePostureMarker, ApplicationFeatureUnavailable,
    ApplicationInputPort, ApplicationOutputPort, ApplicationPortDeclaration, ApplicationPortRef,
    ApplicationProgramFeature, ApplicationProgramFeatureNode, ApplicationProgramFeatureSet,
    ApplicationProgramInputSet, ApplicationProgramOutputSet,
};
pub use identity::ApplicationProgramIdentity;
pub use inventory::{
    ApplicationProgramInventory, ApplicationProgramInventoryDeclaration,
    ApplicationProgramInventoryIdentity, ApplicationProgramInventoryNode,
    ApplicationProgramInventorySet, ApplicationProgramOutput, ApplicationProgramOutputDeclaration,
    ApplicationProgramOutputInventorySet, ApplicationProgramOutputNode,
};
pub use program::{
    validate_application_program, ApplicationProgramDefinition, ApplicationProgramValidationDenial,
    ApplicationProgramValidationDenialKind, ValidatedApplicationProgram,
};
pub use rule::{
    ApplicationLocalRuleRef, ApplicationProgramExecutionPoint, ApplicationProgramLocalRule,
    ApplicationProgramRuleDeclaration, ApplicationProgramRuleNode, ApplicationProgramRulePosture,
    ApplicationProgramRuleSet, ApplicationProgramSharedRule,
    ApplicationProgramUnavailableLocalRule, ApplicationProgramUnavailableSharedRule,
    ApplicationSharedRuleRef, AtCommitBoundary, AtMutationSensitive, AtSnapshotPublication,
};
