//! Typed values in an operation body, distinct from its derived effect ceiling.

mod builder;
mod program;
mod validation;

pub use builder::ApplicationOperationBodyBuilder;
pub use program::{ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField};
pub(crate) use validation::validate_operation_bodies;

use std::marker::PhantomData;

use crate::application_schema::ApplicationValueEncodeDenial;

/// Completed authored meaning. Construction validates all typed literal values.
pub struct ApplicationOperationBody<Schema, Operation, Input> {
    pub(super) program: ApplicationOperationBodyProgram,
    pub(super) marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

/// A body must be complete and unambiguous before schema registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationOperationBodyAuthoringDenial {
    Value(ApplicationValueEncodeDenial),
    Empty,
    MultipleScopeEntities,
    DuplicateAssignment,
    DuplicateAttachment,
}

impl std::fmt::Display for ApplicationOperationBodyAuthoringDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "operation body authoring denied: {self:?}")
    }
}

impl std::error::Error for ApplicationOperationBodyAuthoringDenial {}

impl From<ApplicationValueEncodeDenial> for ApplicationOperationBodyAuthoringDenial {
    fn from(value: ApplicationValueEncodeDenial) -> Self {
        Self::Value(value)
    }
}
