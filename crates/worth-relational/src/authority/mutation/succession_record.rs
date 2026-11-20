//! One coupled chain record as a single round of its latest re-derivation
//! would publish it. Folding a successor round re-runs the ordinary record
//! publication from the chain's base, not a patch-by-patch netting of rounds.
use super::canonical_deltas::{canonical_delta_for_mutation, CanonicalRecordAspectDelta};
use super::effect_publication::canonical_publication_fragment;
use super::intents::entity_aspect_preparation::{
    prepare_fields, prepare_portable, PreparedEntityAspectMutation,
};
use super::intents::field_authoring_candidate::FieldAuthoringDomain;
use super::intents::record_aspect_patch;
use super::patch_details::{patch_detail_for_entity, EntityPatchDetailKind};
use super::{FoundationalPatchFragment, RecordMutation};
use crate::identity::data::EntityId;
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::substrate::StorageAllocation;
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MutationIntent, RecordAspectPatchTarget,
    TransactionCommitError,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

/// `intent` is the round's one authored intent on `entity`: the Create of a
/// record the chain allocated, or an update of a record that existed in
/// `base`, the candidate before the chain first wrote it.
pub(crate) fn succession_record(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plans: &AspectContractPlanCatalog,
    base: &impl CurrentPartitionAccess,
    entity: EntityId,
    intent: &MutationIntent,
) -> Result<(CanonicalRecordAspectDelta, FoundationalPatchFragment), TransactionCommitError> {
    let (mutation, kind) = match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            let target = RecordAspectPatchTarget::EntityCreation {
                kind_id: spec.kind_id,
            };
            let patch = record_aspect_patch::readmit_field_authoring(
                execution,
                &spec.fields,
                PortablePatchReadmissionPurpose::RecordCreation,
                plans.entity_plans.get(&spec.kind_id),
                target,
                FieldAuthoringDomain::Entity,
            )?;
            let mutation = RecordMutation::EntityCreated {
                entity_id: entity,
                kind_id: spec.kind_id,
                authoritative_patch: record_aspect_patch::published_patch(patch),
            };
            (mutation, EntityPatchDetailKind::Created)
        }
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
            if update.entity_id == entity =>
        {
            let prepared = prepare_fields(execution, update, base, plans)?;
            (updated(entity, prepared), EntityPatchDetailKind::Updated)
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch))
            if patch.entity_id == entity =>
        {
            let prepared = prepare_portable(execution, patch, base, plans)?;
            (updated(entity, prepared), EntityPatchDetailKind::Updated)
        }
        _ => {
            return Err(crate::mvcc::origin_denial(
                "a folded chain record has no single entity write",
            ))
        }
    };
    let delta = canonical_delta_for_mutation(&mutation, plans)
        .map_err(|error| TransactionCommitError::conflict(error.to_commit_conflict()))?;
    let patch = match mutation {
        RecordMutation::EntityCreated {
            authoritative_patch,
            ..
        }
        | RecordMutation::EntityUpdated {
            authoritative_patch,
            ..
        } => authoritative_patch,
        _ => unreachable!("only entity creates and updates are folded"),
    };
    let fragment =
        canonical_publication_fragment(&delta, patch_detail_for_entity(kind, entity), patch)
            .map_err(TransactionCommitError::conflict)?;
    Ok((delta, fragment))
}

fn updated(entity: EntityId, prepared: PreparedEntityAspectMutation) -> RecordMutation {
    let PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state,
        mut new_authoritative_aspect_state,
        patch,
    } = prepared;
    if let Some(prior) = &authoritative_aspect_state {
        new_authoritative_aspect_state.lineage_id = prior.lineage_id;
        new_authoritative_aspect_state.structural_fingerprint = prior.structural_fingerprint;
    }
    RecordMutation::EntityUpdated {
        entity_id: entity,
        kind_id,
        old_authoritative_aspect_state: authoritative_aspect_state,
        new_authoritative_aspect_state: Some(StorageAllocation::new(
            new_authoritative_aspect_state,
        )),
        authoritative_patch: Some(patch),
    }
}
