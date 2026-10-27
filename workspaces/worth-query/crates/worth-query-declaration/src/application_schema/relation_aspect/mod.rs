//! Real relation-owned scalar contracts; relations never impersonate entities.
mod authoring;
mod definition;
mod field;
mod reference;
mod validation;
pub(super) use validation::validate_closure;

pub use definition::{
    ApplicationRelationAspectDefinition, ApplicationRelationAspectDefinitionBuilder,
    ApplicationRelationAspectDefinitionDenial, ErasedApplicationRelationAspectDefinition,
    WorthQueryPortableRelationAspectParts,
};
pub use field::ApplicationRelationFieldDefinition;
pub use reference::ApplicationRelationFieldRef;
