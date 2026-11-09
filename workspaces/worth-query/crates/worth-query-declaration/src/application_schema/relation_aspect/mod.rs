//! Real relation-owned scalar contracts; relations never impersonate entities.
mod definition;
mod field;
mod validation;
pub(super) use validation::validate_closure;

pub use definition::{
    ApplicationRelationAspectDefinitionDenial, ErasedApplicationRelationAspectDefinition,
    WorthQueryPortableRelationAspectParts,
};
pub use field::ApplicationRelationFieldDefinition;
