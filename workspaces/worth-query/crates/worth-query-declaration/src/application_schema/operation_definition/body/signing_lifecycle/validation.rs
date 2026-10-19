//! Cold readmission and typed finish share the lifecycle's structural law.
use super::super::{
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation as Creation,
    ApplicationOperationBodyCreationLinkDirection as Direction,
};
use crate::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyProgram as Body,
    ApplicationOperationProgramTarget as Target, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationScalarValueBinding,
    ApplicationSchemaMember as Member, ApplicationSigningGraphSubjectProvenanceBinding,
    ApplicationSigningSubjectProvenanceBinding, ApplicationSigningSupersessionBinding,
};
#[cfg(test)]
mod tests;

pub(in crate::application_schema::operation_definition::body) fn validate_creation(
    creation: &Creation,
) -> bool {
    let Some(spec) = creation.signing_lifecycle() else {
        return creation.subject_impact_bound().is_none();
    };
    let Some(signing) = creation.signing() else {
        return false;
    };
    spec.maximum_evidence > 0 && !spec.consequence_relation.is_empty()
        && spec.history_relation != spec.current_relation && spec.current_relation != spec.consequence_relation
        && creation.fields().iter().any(|field| matches!(field, Field::SigningProvenance { field, binding }
            if field == signing.provenance() && (*binding == ApplicationSigningSubjectProvenanceBinding::IDENTITY || *binding == ApplicationSigningGraphSubjectProvenanceBinding::IDENTITY)))
        && creation.links().iter().any(|link| link.relation == spec.history_relation
            && link.from == spec.source_entity && link.to == creation.entity() && link.direction == Direction::FromScope)
        && creation.links().iter().any(|link| link.relation == spec.current_relation
            && link.from == spec.source_entity && link.to == creation.entity() && link.direction == Direction::FromScope)
        && creation.entity() != spec.consequence_field.entity()
        && spec.source_entity != spec.consequence_field.entity()
}

pub(in crate::application_schema::operation_definition::body) fn validate(
    body: &Body,
    members: &[Member],
) -> bool {
    let Some(creation) = body.creation() else {
        return true;
    };
    if !validate_creation(creation) {
        return false;
    }
    let Some(spec) = creation.signing_lifecycle() else {
        return true;
    };
    if body.scope_entity() != Some(spec.source_entity.as_str()) || !validate_subject_writes(body) {
        return false;
    }
    let mut record_fields = 0;
    let mut history = false;
    let mut current = false;
    let mut consequences = false;
    for member in members {
        match member {
            Member::Field { entity, aspect, field, value_type, presence, writable, unit, .. }
                if entity == spec.consequence_field.entity() => {
                record_fields += 1;
                if aspect != spec.consequence_field.aspect() || field != spec.consequence_field.field()
                    || *writable || *presence != Presence::Required || unit.is_some()
                    || value_type != ApplicationSigningSupersessionBinding::IDENTITY.as_str() { return false; }
            }
            Member::Relation { relation, from, to, integrity } if relation == &spec.history_relation || relation == &spec.current_relation => {
                let valid = from == &spec.source_entity && to == creation.entity() && integrity.cross_context_policy() == ApplicationRelationCrossContextPolicy::Forbid
                    && integrity.deletion == ApplicationRelationDeletionPolicy::RetainDanglingForAudit;
                if relation == &spec.history_relation { history = valid; } else { current = valid; }
            }
            Member::Relation { relation, from, to, integrity } if relation == &spec.consequence_relation => {
                consequences = from == creation.entity() && to == spec.consequence_field.entity() && integrity.cross_context_policy() == ApplicationRelationCrossContextPolicy::Forbid
                    && integrity.deletion == ApplicationRelationDeletionPolicy::RetainDanglingForAudit;
            }
            Member::OperationProgram { target: Target::Create { entity } | Target::Delete { entity } | Target::Write { entity, .. }, .. }
                if entity == spec.consequence_field.entity() => return false,
            Member::OperationProgram { operation, target: Target::Link { relation, .. } }
                if relation == &spec.history_relation || relation == &spec.current_relation || relation == &spec.consequence_relation => {
                if relation == &spec.consequence_relation || !members.iter().any(|member| matches!(member,
                    Member::OperationBody { operation: candidate, body: owner }
                        if candidate == operation && owner.creation().is_some_and(|owner|
                            owner.entity() == creation.entity() && owner.signing_lifecycle() == Some(spec)))) { return false; }
            }
            Member::OperationProgram { target: Target::Unlink { relation, .. }, .. }
                if relation == &spec.history_relation || relation == &spec.current_relation || relation == &spec.consequence_relation => return false,
            Member::OperationBody { body: other, .. } if other.creation().is_some_and(|other|
                other.entity() == creation.entity() && other.signing_lifecycle() != Some(spec)) => return false,
            _ => {}
        }
    }
    history && current && consequences && record_fields == 1
}

/// Captures certify the observed preimage, never a source value changed by the same body.
pub(in crate::application_schema::operation_definition::body) fn validate_subject_writes(
    body: &Body,
) -> bool {
    body.creation().is_none_or(|creation| {
        creation.signing_lifecycle().is_none()
            || creation.fields().iter().all(|field| {
                let source = match field {
                    Field::CapturedScopeField {
                        source,
                        role: super::super::ApplicationOperationBodyCaptureRole::Subject,
                        ..
                    } => Some(source),
                    Field::CapturedPathField {
                        source,
                        role: super::super::ApplicationOperationBodyCaptureRole::Subject,
                        ..
                    } if source.path.steps().is_empty() => Some(&source.field),
                    _ => None,
                };
                source.is_none_or(|source| {
                    !body
                        .scope_fields()
                        .iter()
                        .any(|step| step.write_target().is_some() && step.field() == source)
                })
            })
    })
}
