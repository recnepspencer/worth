//! Validate body closure before canonical identity or installation exists.

use std::collections::BTreeSet;

use crate::application_schema::{
    ApplicationFieldPresence, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

use super::ApplicationOperationBodyScopeField;

pub(crate) fn validate_operation_bodies(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        let ApplicationSchemaMember::OperationBody { operation, body } = member else {
            continue;
        };
        if !operations.insert(operation) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
        }
        let scope = body
            .scope_entity()
            .ok_or(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency)?;
        let mut writes = BTreeSet::new();
        for step in body.scope_fields() {
            if step.field().entity() != scope {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            if let ApplicationOperationBodyScopeField::Assign { field, .. } = step {
                if !writes.insert(field) {
                    return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
                }
            }
        }
        if writes.is_empty() {
            return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
        }
        if !members.iter().any(|member| {
            matches!(member, ApplicationSchemaMember::Operation { operation: candidate, .. }
                if candidate == operation)
        }) {
            return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
        }
        for step in body.scope_fields() {
            let locus = step.field();
            let found = members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Field {
                    entity, aspect, field, scalar_family, value_type, presence, writable, ..
                } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                    && value_type == step.binding().as_str()
                    && *scalar_family == step.value().value_family()
                    && *presence == ApplicationFieldPresence::Required
                    && (!matches!(step, ApplicationOperationBodyScopeField::Assign { .. }) || *writable))
            });
            if !found {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            if !members.iter().any(|member| {
                matches!(member,
                ApplicationSchemaMember::OperationDecisionRead { operation: candidate, target }
                    if candidate == operation && target == &step.decision_read())
            }) {
                return Err(
                    ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency,
                );
            }
            if let Some(write) = step.write_target() {
                if !members.iter().any(|member| {
                    matches!(member,
                    ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &write)
                }) {
                    return Err(
                        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                    );
                }
            }
        }
    }
    Ok(())
}
