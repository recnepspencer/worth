use crate::authority::mutation::intents::{field_authoring_candidate, record_aspect_patch};
use crate::capabilities::AspectPlanSource;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::{
    CommitConflict, EntityMutationIntent, EntityReference, RecordAspectPatchTarget,
    RelationMutationIntent,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTransactionRelationValue {
    pub relation_id: crate::identity::data::RelationId,
    pub kind: crate::schema::data::KindResolution,
    pub lifecycle: crate::storage::data::RecordLifecycleState,
    pub created_at_version: crate::identity::data::VersionId,
    pub retired_at_version: Option<crate::identity::data::VersionId>,
    pub source: EntityReference,
    pub target: EntityReference,
    pub authoritative_aspect_state:
        Option<worth_foundational::facade::AuthoritativeRecordAspectState>,
}

impl From<RelationReadRecord> for RelationalTransactionRelationValue {
    fn from(record: RelationReadRecord) -> Self {
        Self {
            relation_id: record.relation_id,
            kind: record.kind,
            lifecycle: record.lifecycle,
            created_at_version: record.created_at_version,
            retired_at_version: record.retired_at_version,
            source: EntityReference::Existing(record.source),
            target: EntityReference::Existing(record.target),
            authoritative_aspect_state: record.authoritative_aspect_state,
        }
    }
}

pub(super) fn project_entity(
    schema: &impl AspectPlanSource,
    mut effective: Option<EntityReadRecord>,
    staged: &[EntityMutationIntent],
) -> Result<Option<EntityReadRecord>, CommitConflict> {
    for intent in staged {
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
                    &intent.fields,
                    PortablePatchReadmissionPurpose::RecordMutation,
                    schema.entity_aspect_plan(record.kind.kind_id),
                    target,
                    field_authoring_candidate::FieldAuthoringDomain::Entity,
                )?;
                record.authoritative_aspect_state = record_aspect_patch::apply(
                    record.authoritative_aspect_state.as_ref(),
                    &patch,
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
                record.authoritative_aspect_state = record_aspect_patch::apply(
                    record.authoritative_aspect_state.as_ref(),
                    &patch,
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

pub(super) fn project_relation(
    schema: &impl AspectPlanSource,
    base: Option<RelationReadRecord>,
    staged: &[RelationMutationIntent],
) -> Result<Option<RelationalTransactionRelationValue>, CommitConflict> {
    let mut effective = base.map(RelationalTransactionRelationValue::from);
    for intent in staged {
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
                record.authoritative_aspect_state = record_aspect_patch::apply(
                    record.authoritative_aspect_state.as_ref(),
                    &patch,
                    target,
                )?;
            }
            RelationMutationIntent::Delete(_) => effective = None,
        }
    }
    Ok(effective)
}
