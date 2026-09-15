//! Authored static application composition.

mod connection;
mod feature;
mod identity;
mod program;
mod rule;

pub use connection::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity, ApplicationConnectionRef,
    ApplicationOccurrenceConnectionBinding,
};
pub use feature::{
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationInputPort, ApplicationOutputPort,
    ApplicationPortRef,
};
pub use identity::ApplicationProgramIdentity;
pub use program::{
    ApplicationProgramAuthoring, ApplicationProgramComplete, ApplicationProgramConnectionRequired,
    ApplicationProgramDefinition, ApplicationProgramDependentConnectionRequired,
    ApplicationProgramLocalRuleRequired, ApplicationProgramSharedRuleRequired,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
    ValidatedApplicationProgram,
};
pub use rule::{
    ApplicationLocalRuleRef, ApplicationProgramRuleDeclaration, ApplicationSharedRuleRef,
};
