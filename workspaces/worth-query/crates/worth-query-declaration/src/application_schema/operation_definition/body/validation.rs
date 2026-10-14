//! Validate body closure before canonical identity or installation exists.

use std::collections::BTreeSet;

use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationProgramTarget,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

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
            if step.write_target().is_some() && !writes.insert(step.field()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
            }
        }
        if let Some(creation) = body.creation() {
            if creation.key().is_empty() || creation.key().trim() != creation.key() {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let creates = ApplicationOperationProgramTarget::Create {
                entity: creation.entity().to_owned(),
            };
            if !members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Entity { entity } if entity == creation.entity())
            }) || !members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                    if candidate == operation && target == &creates)
            }) {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let mut initialized = BTreeSet::new();
            for step in creation.fields() {
                let locus = step.field();
                if locus.entity() != creation.entity() || !initialized.insert(locus.clone()) {
                    return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
                }
                if step
                    .input_member()
                    .is_some_and(|member| member.is_empty() || member.trim() != member)
                {
                    return Err(
                        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                    );
                }
                if !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::Field {
                        entity, aspect, field, scalar_family, value_type, ..
                    } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                        && value_type == step.binding().as_str()
                        && *scalar_family == step.scalar_family())
                }) || !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &step.write_target())
                }) {
                    return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
                }
            }
            for member in members {
                if let ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field,
                    presence: ApplicationFieldPresence::Required,
                    ..
                } = member
                {
                    if entity == creation.entity()
                        && !initialized.iter().any(|locus| {
                            locus.entity() == entity
                                && locus.aspect() == aspect
                                && locus.field() == field
                        })
                    {
                        return Err(
                            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                        );
                    }
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
            if step
                .input_member()
                .is_some_and(|member| member.is_empty() || member.trim() != member)
            {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let found = members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Field {
                    entity, aspect, field, scalar_family, value_type, presence, writable, ..
                } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                    && value_type == step.binding().as_str()
                    && *scalar_family == step.scalar_family()
                    && *presence == ApplicationFieldPresence::Required
                    && (step.write_target().is_none() || *writable))
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
