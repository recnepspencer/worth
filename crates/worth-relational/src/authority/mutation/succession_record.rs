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
use crate::transactions::data::intents::CreateIntent;
use crate::transactions::data::intents::EntityMutationIntent;
use crate::transactions::data::intents::MutationIntent;
use crate::transactions::data::outcomes::conflict_class::RecordAspectPatchTarget;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
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
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "a folded chain record has no single entity write",
                ),
            )
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

/// Reset `entity`'s revision maps in `state` (and `delta_state` when the
/// round materialized that slot) to `base`'s pre-chain maps (empty for
/// `Created`), then re-stamp the folded `delta` at `version`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn restamp_succession_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut crate::storage::overlay::WorkingState,
    delta_state: &mut crate::storage::overlay::WorkingState,
    symbols: &mut dyn crate::runtime::state::subsystems::services::symbol_table::MutationSymbolMeanings,
    base: &impl CurrentPartitionAccess,
    delta: &CanonicalRecordAspectDelta,
    entity: EntityId,
    version: crate::identity::data::VersionId,
) -> Result<(), super::MutationApplyDenial> {
    use crate::publication::patch::data::RecordStructuralChange;
    use crate::storage::substrate::{
        partition_of, slot_of, AspectVersions, EntityRecordKind, FieldRevisions,
    };

    let partition_id = partition_of::<EntityRecordKind>(&entity);
    let slot = slot_of::<EntityRecordKind>(&entity);
    let (base_aspects, base_fields) = if delta.structural_change == RecordStructuralChange::Created
    {
        (AspectVersions::new(), Some(FieldRevisions::new()))
    } else {
        let base_partition = base.get_current_partition(partition_id);
        let aspects = base_partition
            .as_ref()
            .and_then(|p| p.entity_arena().aspect_versions_at(slot))
            .cloned()
            .unwrap_or_default();
        let fields = base_partition
            .as_ref()
            .and_then(|p| p.entity_arena().field_revisions_at(slot))
            .cloned();
        (aspects, fields)
    };
    let stop = |cause: crate::execution::RelationalExecutionDenialCause| {
        super::MutationApplyDenial::Commit(TransactionCommitError::execution(cause.into()))
    };
    state
        .get_partition_mut_for_request(partition_id, execution)
        .map_err(stop)?
        .entity_arena
        .replace_revision_maps_for_request(slot, base_aspects, base_fields, execution)
        .map_err(stop)?;
    let admitted = super::delta_symbols::AdmittedDeltaSymbols::admit(delta, symbols)?;
    super::aspect_versions::write_aspect_versions_for_delta_for_request(
        execution, state, &admitted, version,
    )
    .map_err(stop)?;
    super::field_versions::write_field_versions_for_delta(execution, state, &admitted, version)
        .map_err(stop)?;
    // Share the freshly folded persistent revision maps with `delta_state`
    // when this round materialized the slot, so `advance_pin` publishes the
    // same folded stamps into the candidate pin.
    if delta_state
        .get_current_partition(partition_id)
        .and_then(|p| p.entity_arena().aspect_versions_at(slot))
        .is_some()
    {
        let partition = state
            .get_current_partition(partition_id)
            .expect("folded entity partition was just stamped in state");
        let aspects = partition
            .entity_arena()
            .aspect_versions_at(slot)
            .cloned()
            .unwrap_or_default();
        let fields = partition.entity_arena().field_revisions_at(slot).cloned();
        delta_state
            .get_partition_mut_for_request(partition_id, execution)
            .map_err(stop)?
            .entity_arena
            .replace_revision_maps_for_request(slot, aspects, fields, execution)
            .map_err(stop)?;
    }
    Ok(())
}
