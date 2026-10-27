//! Rebuildable relation and collection meaning, separate from physical records.
mod authoring;
mod collection;
mod definition;
mod denial;
mod expression;
mod expression_validation;
mod recipe;
mod reference;
mod source_closure;
pub use source_closure::{
    application_derived_relation_physical_sources, application_derived_relation_source_closure,
    ApplicationDerivedRelationSourceClosure,
};
mod validation;
pub(super) use validation::{validate_closure, validate_member_identifiers};

pub use collection::ApplicationRuleReadableCollectionAssociation;
pub use definition::{
    ErasedApplicationDerivedRelationDefinition, WorthQueryPortableDerivedRelationParts,
};
pub use denial::ApplicationDerivedRelationDefinitionDenial;
pub use expression::{
    ApplicationDerivedRelationExpression, ApplicationDerivedRelationNode,
    ApplicationDerivedRelationSource, ApplicationDerivedRelationSourceKind,
};
pub use recipe::{ApplicationDerivedRelation, ApplicationDerivedRelationOperand};
pub use reference::ApplicationDerivedRelationRef;
