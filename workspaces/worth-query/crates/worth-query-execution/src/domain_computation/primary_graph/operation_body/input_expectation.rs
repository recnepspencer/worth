//! Compare already admitted captured members with tracked scope facts before effects.
use super::super::{DecisionReader, HandlerResult};
use super::{BodyFieldExpectation, PreparedBodyInputs, WorthQueryOperationBodyDenial};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField, ApplicationSchema,
    },
};

pub(super) fn evaluate<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    body: &ApplicationOperationBodyProgram,
    inputs: Option<&PreparedBodyInputs>,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
    let member = body
        .scope_fields()
        .iter()
        .find_map(|step| match step {
            ApplicationOperationBodyScopeField::ExpectInput { member, .. } => Some(member),
            _ => None,
        })
        .or_else(|| {
            body.path_input_expectations()
                .first()
                .map(|guard| &guard.member)
        });
    let Some(member) = member else {
        return HandlerResult::Completed(());
    };
    let Some(inputs) = inputs else {
        return HandlerResult::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch {
            member: member.clone(),
        });
    };
    for (step, value) in body
        .scope_fields()
        .iter()
        .filter(|step| step.input_member().is_some())
        .zip(&inputs.values)
    {
        let ApplicationOperationBodyScopeField::ExpectInput { field, .. } = step else {
            continue;
        };
        let expected = value.as_ref().expect("required input guard was admitted");
        match reader.body_scope_field(field, BodyFieldExpectation::Equals(expected)) {
            Ok(true) => {}
            Ok(false) => {
                return HandlerResult::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch {
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                })
            }
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
    }
    let start = body
        .scope_fields()
        .iter()
        .filter(|step| step.input_member().is_some())
        .count()
        + body
            .creation()
            .into_iter()
            .flat_map(|creation| creation.fields())
            .filter(|step| step.input_member().is_some())
            .count();
    for (guard, value) in body
        .path_input_expectations()
        .iter()
        .zip(&inputs.values[start..])
    {
        let expected = value
            .as_ref()
            .expect("required path input guard was admitted");
        match reader.body_path_input_equals(&guard.source, expected) {
            Ok(true) => {}
            Ok(false) => {
                let field = &guard.source.field;
                return HandlerResult::DomainDenied(
                    WorthQueryOperationBodyDenial::SourceMismatch {
                        entity: field.entity().into(),
                        aspect: field.aspect().into(),
                        field: field.field().into(),
                    },
                );
            }
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
    }
    HandlerResult::Completed(())
}
