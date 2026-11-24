pub mod work_observation;
use crate::authority::mutation::intents::{field_authoring_candidate, record_aspect_patch};
use crate::contracts::schema::aspect_plan_source::AspectPlanSource;
use crate::contracts::storage::data::read_records::EntityReadRecord;
use crate::contracts::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, RelationMutationIntent,
};
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

pub use super::relation_value::RelationalTransactionRelationValue;

pub fn project_entity<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    schema: &impl AspectPlanSource,
    mut effective: Option<EntityReadRecord>,
    staged: impl IntoIterator<Item = &'a EntityMutationIntent>,
) -> Result<Option<EntityReadRecord>, worth_relational_branch::internal_port::mvcc::transaction::read_denial::RelationalTransactionReadDenial>{
    for intent in staged {
        work_observation::record();
        match intent {
            EntityMutationIntent::UpdateFields(intent) => {
                let Some(record) = effective.as_mut() else {
                    continue;
                };
                let target = RecordAspectPatchTarget::Entity {
                    entity_id: intent.entity_id,
                    kind_id: record.kind.kind_id,
                };
                let patch = record_aspect_patch::readmit_field_authoring(
                    execution,
                    &intent.fields,
                    PortablePatchReadmissionPurpose::RecordMutation,
                    schema.entity_aspect_plan(record.kind.kind_id),
                    target,
                    field_authoring_candidate::FieldAuthoringDomain::Entity,
                )?;
                record.authoritative_aspect_state = apply_projection_state(
                    record.authoritative_aspect_state.as_ref(),
                    patch.as_patch(),
                    target,
                )?;
            }
            EntityMutationIntent::ApplyAspectPatch(intent) => {
                let Some(record) = effective.as_mut() else {
                    continue;
                };
                let target = RecordAspectPatchTarget::Entity {
                    entity_id: intent.entity_id,
                    kind_id: record.kind.kind_id,
                };
                let patch = record_aspect_patch::readmit(
                    intent.aspect_patch.clone(),
                    PortablePatchReadmissionPurpose::RecordMutation,
                    schema.entity_aspect_plan(record.kind.kind_id),
                    target,
                )?;
                record.authoritative_aspect_state = apply_projection_state(
                    record.authoritative_aspect_state.as_ref(),
                    patch.as_patch(),
                    target,
                )?;
            }
            EntityMutationIntent::Replace(_) | EntityMutationIntent::Delete(_) => effective = None,
            // A revalidation demand changes nothing, so the effective record is
            // exactly what the preceding staged intents left.
            EntityMutationIntent::Revalidate(_) => {}
        }
    }
    Ok(effective)
}

pub fn project_relation<'a>(
    schema: &impl AspectPlanSource,
    base: Option<RelationalTransactionRelationValue>,
    staged: impl IntoIterator<Item = &'a RelationMutationIntent>,
) -> Result<Option<RelationalTransactionRelationValue>, CommitConflict> {
    let mut effective = base;
    for intent in staged {
        work_observation::record();
        match intent {
            RelationMutationIntent::UpdateEndpoints(intent) => {
                let Some(record) = effective.as_mut() else {
                    continue;
                };
                record.source = intent.source.clone();
                record.target = intent.target.clone();
            }
            RelationMutationIntent::ApplyAspectPatch(intent) => {
                let Some(record) = effective.as_mut() else {
                    continue;
                };
                let target = RecordAspectPatchTarget::Relation {
                    relation_id: intent.relation_id,
                    kind_id: record.kind.kind_id,
                };
                let patch = record_aspect_patch::readmit(
                    intent.aspect_patch.clone(),
                    PortablePatchReadmissionPurpose::RecordMutation,
                    schema.relation_aspect_plan(record.kind.kind_id),
                    target,
                )?;
                record.authoritative_aspect_state = apply_projection_state(
                    record.authoritative_aspect_state.as_ref(),
                    patch.as_patch(),
                    target,
                )?;
            }
            RelationMutationIntent::Revalidate(_) => {}
            RelationMutationIntent::Delete(_) => effective = None,
        }
    }
    Ok(effective)
}

/// Independent ordinary read-record copies; no Native extra or retained bill moves here.
fn apply_projection_state(
    current: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<Option<worth_foundational::facade::AuthoritativeRecordAspectState>, CommitConflict> {
    match patch.apply_to_optional(current) {
        worth_proof::TransitionOutcome::Success(state) => {
            Ok(state.map(|value| value.into_parts().into_parts().0))
        }
        worth_proof::TransitionOutcome::Denied(denial) => Err(record_aspect_patch::conflict(
            target,
            crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchDenial::ApplicationDenied(denial),
        )),
    }
}
