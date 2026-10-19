//! Detach one exact settlement; physical keys leave only with the final binding.
use super::super::admission::PreparedTreeEdits;
use super::*;

pub(super) fn remove(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    projection: &Arc<PostingProjection>,
    admission: &mut impl RetainedIndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
    let id = projection.id;
    admission.ordered_read(state.posting_bindings.len())?;
    let Some(existing) = state.posting_bindings.get(&id) else {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    };
    let existing = existing
        .settlements()
        .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
    if !Arc::ptr_eq(&existing.projection, projection) || !existing.settlements.contains(identity) {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    }
    let mut binding = existing.clone();
    admission.index_remove::<Arc<RecordedSettlementIdentity>, ()>(binding.settlements.len())?;
    binding.settlements.remove(identity);
    if !binding.settlements.is_empty() {
        admission.index_bytes(
            index_capacity::arc_bytes::<PostingBindings>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.index_edit::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
        state
            .posting_bindings
            .insert(id, Arc::new(PostingBindings::Settlements(binding)));
        return Ok(());
    }
    remove_physical(state, id, &projection.ordinals, admission)
}

/// Remove exactly one physical reader projection; retained historical roots keep theirs.
pub(in super::super) fn remove_physical(
    state: &mut MarkState,
    id: u64,
    keys: &im::OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    admission: &mut impl RetainedIndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<u64>>::new();
    for key in keys.keys() {
        detach_key(state, id, key, &mut map_edits, admission)?;
    }
    map_edits.retain(state.postings.len(), admission)?;
    admission.index_remove::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
    state.posting_bindings.remove(&id);
    Ok(())
}

/// Deferred read cleanup owns only the touched bucket, never its old footprint.
pub(in super::super) fn remove_physical_key(
    state: &mut MarkState,
    id: u64,
    key: &FactPostingKey,
    admission: &mut impl RetainedIndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<u64>>::new();
    detach_key(state, id, key, &mut map_edits, admission)?;
    map_edits.retain(state.postings.len(), admission)
}

fn detach_key(
    state: &mut MarkState,
    id: u64,
    key: &FactPostingKey,
    map_edits: &mut PreparedTreeEdits<Arc<FactPostingKey>, OrdSet<u64>>,
    admission: &mut impl RetainedIndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
    admission.key_read(key, state.postings.len())?;
    let Some((stored_key, groups)) = state.postings.get_key_value(key) else {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    };
    let stored_key = Arc::clone(stored_key);
    let mut groups = groups.clone();
    let mut edits = PreparedTreeEdits::<u64, ()>::new();
    edits.remove(groups.len(), admission)?;
    if groups.remove(&id).is_none() {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    }
    debit(&mut state.posting_count, 1)?;
    if groups.is_empty() {
        map_edits.key_remove(key, state.postings.len(), admission)?;
        state.postings.remove(key);
        debit(
            &mut state.key_payload_bytes,
            super::postings::key_payload_bytes(key)?,
        )?;
    } else {
        edits.retain(groups.len(), admission)?;
        map_edits.key_insert(key, state.postings.len(), admission)?;
        state.postings.insert(stored_key, groups);
    }
    Ok(())
}
