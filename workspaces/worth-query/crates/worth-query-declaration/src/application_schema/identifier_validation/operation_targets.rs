//! Identifier validation for declared operation decision and program targets.
use super::{validate_identifiers, validate_simple_identifier, ApplicationSchemaDeclarationDenial};
use crate::application_schema::schema_member::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
};

pub(super) fn validate_decision_read_target_identifiers(
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

pub(super) fn validate_program_target_identifiers(
    target: &ApplicationOperationProgramTarget,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match target {
        ApplicationOperationProgramTarget::RemoveAspect { entity, aspect } => {
            validate_identifiers([entity, aspect])
        }
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
        ApplicationOperationProgramTarget::InitializeRelationField {
            relation,
            from,
            to,
            aspect,
            field,
        } => validate_identifiers([relation, from, to, aspect, field]),
    }
}
