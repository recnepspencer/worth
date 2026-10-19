//! Literal scope expectations retain their native facts before candidate preparation.
use super::super::BodyFieldExpectation;
use crate::domain_computation::primary_graph::{
    DecisionReader, HandlerResult, WorthQueryOperationBodyDenial,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField, ApplicationSchema,
    },
};
pub(super) fn evaluate<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    body: &ApplicationOperationBodyProgram,
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
    for step in body.scope_fields() {
        if matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
                | ApplicationOperationBodyScopeField::ExpectInput { .. }
        ) {
            continue;
        }
        let expected = match step {
            ApplicationOperationBodyScopeField::Expect { value, .. } => {
                BodyFieldExpectation::Equals(value)
            }
            ApplicationOperationBodyScopeField::RequirePresent { .. } => {
                BodyFieldExpectation::Present
            }
            ApplicationOperationBodyScopeField::ExpectInput { .. } => {
                unreachable!("input guards compare prepared canonical members")
            }
            ApplicationOperationBodyScopeField::Assign { .. }
            | ApplicationOperationBodyScopeField::AssignValue { .. }
            | ApplicationOperationBodyScopeField::AssignInput { .. }
            | ApplicationOperationBodyScopeField::SetOptionalInput { .. }
            | ApplicationOperationBodyScopeField::AssignOptionalInput { .. }
            | ApplicationOperationBodyScopeField::ClearOptional { .. } => BodyFieldExpectation::Any,
        };
        let matches = match reader.body_scope_field(step.field(), expected) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if !matches {
            let field = step.field();
            return HandlerResult::DomainDenied(
                if matches!(
                    step,
                    ApplicationOperationBodyScopeField::RequirePresent { .. }
                ) {
                    WorthQueryOperationBodyDenial::RequirementUnmet {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                    }
                } else {
                    WorthQueryOperationBodyDenial::SourceMismatch {
                        entity: field.entity().to_owned(),
                        aspect: field.aspect().to_owned(),
                        field: field.field().to_owned(),
                    }
                },
            );
        }
    }
    HandlerResult::Completed(())
}
