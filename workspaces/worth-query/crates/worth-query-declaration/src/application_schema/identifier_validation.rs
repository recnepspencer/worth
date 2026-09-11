use super::authorization_policy::ApplicationAuthorizationPath;
use super::capability_identifier_validation::validate_capability_identifiers;
use super::declaration_denial::ApplicationSchemaDeclarationDenial;
use super::schema_member::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    ApplicationSchemaMember,
};

mod application_query;

pub(super) fn validate_schema_header(
    owner: &str,
    name: &str,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if owner.starts_with('.') || owner.ends_with('.') || owner.contains("..") {
        return Err(ApplicationSchemaDeclarationDenial::InvalidIdentifier);
    }
    for segment in owner.split('.') {
        validate_simple_identifier(segment)?;
    }
    validate_simple_identifier(name)?;
    Ok(())
}

pub(super) fn validate_simple_identifier(
    value: &str,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(char::is_whitespace)
        || value.contains('.')
    {
        return Err(ApplicationSchemaDeclarationDenial::InvalidIdentifier);
    }
    Ok(())
}

pub(super) fn validate_member_identifiers(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for member in members {
        match member {
            ApplicationSchemaMember::Entity { entity } => validate_simple_identifier(entity)?,
            ApplicationSchemaMember::Aspect { entity, aspect, .. } => {
                validate_identifiers([entity, aspect])?;
            }
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                unit,
                ..
            } => {
                validate_identifiers([entity, aspect, field])?;
                if let Some(unit) = unit {
                    validate_simple_identifier(unit)?;
                }
            }
            ApplicationSchemaMember::Relation {
                relation, from, to, ..
            } => {
                validate_identifiers([relation, from, to])?;
            }
            ApplicationSchemaMember::PrincipalBinding {
                binding,
                mapping_entity,
                identity_aspect,
                identity_field,
                status_aspect,
                status_field,
                target_relation,
                principal_entity,
                principal_identity_aspect,
                principal_identity_field,
                ..
            } => validate_identifiers([
                binding,
                mapping_entity,
                identity_aspect,
                identity_field,
                status_aspect,
                status_field,
                target_relation,
                principal_entity,
                principal_identity_aspect,
                principal_identity_field,
            ])?,
            ApplicationSchemaMember::ApplicationQuery { definition } => {
                application_query::validate(definition)?;
            }
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                validate_capability_identifiers(contract)?;
            }
            ApplicationSchemaMember::ApplicationCapabilityContext {
                context,
                context_type,
            } => {
                validate_simple_identifier(context)?;
                validate_portable_type_identity(context_type)?;
            }
            ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
                context,
                context_type,
                slot,
                slot_type,
                entity,
            } => {
                validate_identifiers([context, slot, entity])?;
                validate_portable_type_identity(context_type)?;
                validate_portable_type_identity(slot_type)?;
            }
            ApplicationSchemaMember::ApplicationCapabilityProvenance {
                provenance,
                provenance_type,
            } => {
                validate_simple_identifier(provenance)?;
                validate_portable_type_identity(provenance_type)?;
            }
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } => {
                validate_simple_identifier(operation)?;
                validate_portable_type_identity(input_type)?;
            }
            ApplicationSchemaMember::OperationProgram { operation, target } => {
                validate_simple_identifier(operation)?;
                validate_program_target_identifiers(target)?;
            }
            ApplicationSchemaMember::OperationDecisionRead { operation, target } => {
                validate_simple_identifier(operation)?;
                validate_decision_read_target_identifiers(target)?;
            }
            ApplicationSchemaMember::OperationMutationPrecondition { operation, target } => {
                validate_simple_identifier(operation)?;
                validate_simple_identifier(target.entity())?;
                validate_simple_identifier(target.aspect())?;
                validate_simple_identifier(target.field_name())?;
            }
            ApplicationSchemaMember::OperationDecisionFactBudget { operation, .. } => {
                validate_simple_identifier(operation)?;
            }
            ApplicationSchemaMember::OperationProjectionWorkBudget { operation, .. } => {
                validate_simple_identifier(operation)?;
            }
            ApplicationSchemaMember::OperationExternalEffect {
                operation,
                effect,
                rust_payload_type,
                ..
            } => {
                validate_identifiers([operation, effect])?;
                validate_portable_type_identity(rust_payload_type)?;
            }
            ApplicationSchemaMember::OperationAftermath { operation, .. } => {
                validate_simple_identifier(operation)?;
            }
            ApplicationSchemaMember::Policy { policy } => validate_simple_identifier(policy)?,
            ApplicationSchemaMember::Ability {
                ability,
                scope_entity,
            } => validate_identifiers([ability, scope_entity])?,
            ApplicationSchemaMember::OperationAbility {
                operation,
                ability,
                scope_entity,
            } => validate_identifiers([operation, ability, scope_entity])?,
            ApplicationSchemaMember::AbilityPolicy {
                ability,
                scope_entity,
                policy,
                paths,
            } => {
                validate_identifiers([ability, scope_entity, policy])?;
                for path in paths {
                    validate_authorization_path(path)?;
                }
            }
            ApplicationSchemaMember::Unit { unit } => {
                validate_simple_identifier(unit)?;
            }
            ApplicationSchemaMember::Effect {
                effect,
                payload_type,
            } => {
                validate_simple_identifier(effect)?;
                validate_portable_type_identity(payload_type)?;
            }
        }
    }
    Ok(())
}

fn validate_portable_type_identity(
    identity: &crate::portable_identity::WorthQueryPortableTypeIdentity,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if identity.is_valid() {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::InvalidIdentifier)
    }
}

pub(super) fn validate_portable_type_identifier(
    value: &str,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if !value.is_empty()
        && value.trim() == value
        && !value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::InvalidIdentifier)
    }
}

pub(super) fn validate_authorization_path(
    path: &ApplicationAuthorizationPath,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_simple_identifier(path.principal_entity())?;
    validate_simple_identifier(path.scope_entity())?;
    for traversal in path.traversals() {
        validate_simple_identifier(traversal.relation())?;
        validate_simple_identifier(traversal.from())?;
        validate_simple_identifier(traversal.to())?;
    }
    for predicate in path.predicates() {
        validate_simple_identifier(predicate.entity())?;
        validate_simple_identifier(predicate.aspect())?;
        validate_simple_identifier(predicate.field())?;
    }
    Ok(())
}

fn validate_decision_read_target_identifiers(
    target: &ApplicationOperationDecisionReadTarget,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match target {
        ApplicationOperationDecisionReadTarget::Entity { entity } => {
            validate_simple_identifier(entity)
        }
        ApplicationOperationDecisionReadTarget::Field {
            entity,
            aspect,
            field,
        } => validate_identifiers([entity, aspect, field]),
        ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
            validate_identifiers([relation, from, to])
        }
    }
}

fn validate_program_target_identifiers(
    target: &ApplicationOperationProgramTarget,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match target {
        ApplicationOperationProgramTarget::Create { entity }
        | ApplicationOperationProgramTarget::Delete { entity } => {
            validate_simple_identifier(entity)
        }
        ApplicationOperationProgramTarget::Write {
            entity,
            aspect,
            field,
        } => validate_identifiers([entity, aspect, field]),
        ApplicationOperationProgramTarget::Link { relation, from, to }
        | ApplicationOperationProgramTarget::Unlink { relation, from, to } => {
            validate_identifiers([relation, from, to])
        }
        ApplicationOperationProgramTarget::Emit { effect } => validate_simple_identifier(effect),
    }
}

fn validate_identifiers<'a>(
    identifiers: impl IntoIterator<Item = &'a String>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for identifier in identifiers {
        validate_simple_identifier(identifier)?;
    }
    Ok(())
}
