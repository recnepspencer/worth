//! Every computed sink names one installed typed value, not a copied expression.
use crate::application_schema::{
    ApplicationOperationBodyCreatedField as Created, ApplicationOperationBodyProgram as Body,
    ApplicationOperationBodyScopeField as Scope, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
use std::collections::BTreeSet;
pub(in crate::application_schema::operation_definition::body) fn validate_values(
    body: &Body,
    members: &[Member],
) -> Result<(), Denial> {
    let mut names = BTreeSet::new();
    for value in body.values() {
        if !names.insert(value.identity()) {
            return Err(Denial::DuplicateMember);
        }
        let sinks = body
            .scope_fields()
            .iter()
            .filter_map(|step| match step {
                Scope::AssignValue {
                    field,
                    binding,
                    value: name,
                } if name == value.identity() => Some((field, binding)),
                _ => None,
            })
            .chain(
                body.creation()
                    .into_iter()
                    .flat_map(|creation| creation.fields())
                    .filter_map(|step| match step {
                        Created::Computed {
                            field,
                            binding,
                            value: name,
                        } if name == value.identity() => Some((field, binding)),
                        _ => None,
                    }),
            )
            .collect::<Vec<_>>();
        if sinks.is_empty() || sinks.iter().any(|(field,binding)| *binding != value.binding() || !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field: name, value_type, scalar_family: worth_foundational::facade::ScalarAspectType::UInt64, unit: None, .. } if entity == field.entity() && aspect == field.aspect() && name == field.field() && value_type == binding.as_str()))) {
            return Err(Denial::MissingOperationProgramDependency);
        }
    }
    let references = body
        .scope_fields()
        .iter()
        .filter_map(|step| match step {
            Scope::AssignValue { value, .. } => Some(value),
            _ => None,
        })
        .chain(
            body.creation()
                .into_iter()
                .flat_map(|creation| creation.fields())
                .filter_map(|step| match step {
                    Created::Computed { value, .. } => Some(value),
                    _ => None,
                }),
        );
    if references
        .into_iter()
        .any(|name| !names.contains(name.as_str()))
    {
        return Err(Denial::MissingOperationProgramDependency);
    }
    Ok(())
}
