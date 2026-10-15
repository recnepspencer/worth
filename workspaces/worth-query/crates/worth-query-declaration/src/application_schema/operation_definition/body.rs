//! Typed values in an operation body, distinct from its derived effect ceiling.

mod builder;
mod creation;
mod output;
mod output_validation;
mod program;
mod requirement;
mod validation;

pub use builder::ApplicationOperationBodyBuilder;
pub use creation::ApplicationOperationBodyCreationBuilder;
pub use output::{ApplicationOperationBodyOutput, ApplicationOperationBodyOutputTarget};
pub use program::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyCreation,
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};
pub use requirement::{
    ApplicationOperationBodyRequirement, ApplicationOperationBodyRequirementDenial,
    ApplicationOperationBodyRequirementDirection, ApplicationOperationBodyRequirementExpression,
    ApplicationOperationBodyRequirementOperand, ApplicationOperationBodyRequirementOperands,
    ApplicationOperationBodyRequirementProgram, ApplicationOperationBodyRequirementSource,
};
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
    InvalidInputMember,
    InvalidCreationKey,
    IncompatibleSourceBinding,
    ForeignSourceEntity,
    IncompatibleOutput,
    Requirement(ApplicationOperationBodyRequirementDenial),
}

impl From<ApplicationOperationBodyRequirementDenial> for ApplicationOperationBodyAuthoringDenial {
    fn from(value: ApplicationOperationBodyRequirementDenial) -> Self {
        Self::Requirement(value)
    }
}
impl From<worth_foundational::expression_api::ExpressionDenial>
    for ApplicationOperationBodyAuthoringDenial
{
    fn from(value: worth_foundational::expression_api::ExpressionDenial) -> Self {
        Self::Requirement(ApplicationOperationBodyRequirementDenial::Expression(value))
    }
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
