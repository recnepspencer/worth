//! An operation's member visibility meaning, separate from persistent set identity.
mod definition;
mod meaning;
mod validation;

pub use definition::ApplicationOperationComponentReadDefinition;
pub use meaning::{
    ApplicationOperationComponentReadMeaning, WorthQueryPortableOperationComponentReadParts,
};
pub(super) use validation::{validate_closure, validate_identifiers};
