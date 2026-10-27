//! Validate body closure before canonical identity or installation exists.

use std::collections::BTreeSet;

use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyScopeField,
    ApplicationOperationProgramTarget, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

pub(crate) fn validate_operation_bodies(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        let ApplicationSchemaMember::OperationBody { operation, body } = member else {
            continue;
        };
        if !body.has_only_entity_field_loci() {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        if !operations.insert(operation) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
        }
        let scope = body.scope_entity().ok_or_else(|| {
            ApplicationSchemaDeclarationDenial::program_body_dependency(operation)
        })?;
        if !members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::Entity { entity } if entity == scope
        )) || (body.scope_fields().is_empty() && !members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead { operation: candidate, target:
                crate::application_schema::ApplicationOperationDecisionReadTarget::Entity { entity } }
                if candidate == operation && entity == scope
        ))) {
            return Err(ApplicationSchemaDeclarationDenial::decision_read_body_dependency(operation));
        }
        super::requirement::validate_requirements(operation, body, members)?;
        super::retirement::validate(operation, body, members)?;
        super::path_input::validate(operation, body, members)?;
        super::path_value::validate(operation, body, members)?;
        super::reference_input::validate(operation, body, members)?;
        super::emission::validate_emission(operation, body, members)?;
        super::value::validate_values(operation, body, members)?;
        super::output_validation::validate_outputs(operation, body, members)?;
        if !super::enrollment::valid(body) {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        if !super::succession::valid(body) || !super::succession::readmit(body, members) {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        super::principal_validation::validate_principals(operation, body, members)?;
        super::creation_link_validation::validate_links(operation, body, members)?;
        super::creation_reuse::validate(operation, body, members)?;
        super::child_validation::validate(operation, body, members)?;
        super::peer_validation::validate(operation, body, members)?;
        if !super::signing_lifecycle::validate(body, members) {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        let mut writes = BTreeSet::new();
        for step in body.scope_fields() {
            if step.field().entity() != scope {
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                    operation,
                ));
            }
            if step.write_target().is_some() && !writes.insert(step.field()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
            }
        }
        for creation in body
            .creation()
            .into_iter()
            .flat_map(|root| root.initialization_sites())
        {
            if creation.key().is_empty() || creation.key().trim() != creation.key() {
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                    operation,
                ));
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
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(operation));
            }
            let mut initialized = BTreeSet::new();
            for step in creation.fields() {
                let locus = step.field();
                if step.input_is_optional() && !members.iter().any(|member| matches!(member,
                    ApplicationSchemaMember::Field {
            canonical: None, entity, aspect, field, presence: ApplicationFieldPresence::Optional, .. }
                    if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                )) {
                    return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(operation));
                }
                if locus.entity() != creation.entity() || !initialized.insert(locus.clone()) {
                    return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
                }
                if step
                    .input_member()
                    .is_some_and(|member| member.is_empty() || member.trim() != member)
                {
                    return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                        operation,
                    ));
                }
                super::source_validation::validate(operation, scope, step, members)?;
                if !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::Field {
            canonical: None,
                        entity, aspect, field, scalar_family, value_type, ..
                    } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                        && value_type == step.binding().as_str()
                        && *scalar_family == step.scalar_family())
                }) || !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &step.write_target())
                }) {
                    return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(operation));
                }
            }
            for member in members {
                if let ApplicationSchemaMember::Field {
                    canonical: None,
                    entity,
                    aspect,
                    field,
                    presence: ApplicationFieldPresence::Required,
                    ..
                } = member
                    && entity == creation.entity()
                    && !initialized.iter().any(|locus| {
                        locus.entity() == entity
                            && locus.aspect() == aspect
                            && locus.field() == field
                    })
                {
                    return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                        operation,
                    ));
                }
            }
        }
        if writes.is_empty()
            && body.reference_inputs().is_empty()
            && body.path_value_assignments().is_empty()
            && body.creation().is_none()
            && body.outputs().is_empty()
            && body.external_emission().is_none()
        {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        if !members.iter().any(|member| {
            matches!(member, ApplicationSchemaMember::Operation { operation: candidate, .. }
                if candidate == operation)
        }) {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
        for step in body.scope_fields() {
            let locus = step.field();
            let expected_presence = if matches!(
                step,
                ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::AssignOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::ClearOptional { .. }
                    | ApplicationOperationBodyScopeField::RequirePresent { .. }
            ) {
                ApplicationFieldPresence::Optional
            } else {
                ApplicationFieldPresence::Required
            };
            if step
                .input_member()
                .is_some_and(|member| member.is_empty() || member.trim() != member)
            {
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                    operation,
                ));
            }
            let found = members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Field {
            canonical: None,
                    entity, aspect, field, scalar_family, value_type, presence, writable, ..
                } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                    && value_type == step.binding().as_str()
                    && *scalar_family == step.scalar_family()
                    && (matches!(step, ApplicationOperationBodyScopeField::ExpectInput { .. }) || *presence == expected_presence)
                    && (step.write_target().is_none() || *writable))
            });
            if !found {
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                    operation,
                ));
            }
            if !members.iter().any(|member| {
                matches!(member,
                ApplicationSchemaMember::OperationDecisionRead { operation: candidate, target }
                    if candidate == operation && target == &step.decision_read())
            }) {
                return Err(
                    ApplicationSchemaDeclarationDenial::decision_read_body_dependency(operation),
                );
            }
            if let Some(write) = step.write_target()
                && !members.iter().any(|member| {
                    matches!(member,
                    ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &write)
                })
            {
                return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                    operation,
                ));
            }
        }
    }
    super::signing_validation::validate_signed_creations(members)
}

#[cfg(test)]
mod tests;
