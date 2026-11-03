//! Detach one exact settlement; physical keys leave only with the final binding.

use super::*;

pub(super) fn remove(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    projection: &Arc<PostingProjection>,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
    let id = projection.id;

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

    binding.settlements.remove(identity);
    if !binding.settlements.is_empty() {
        state
            .posting_bindings
            .insert(id, Arc::new(PostingBindings::Settlements(binding)));
        return Ok(());
    }
    remove_physical(state, id, &projection.ordinals)
}

/// Remove exactly one physical reader projection; retained historical roots keep theirs.
pub(in super::super) fn remove_physical(
    state: &mut MarkState,
    id: u64,
    keys: &im::OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
) -> Result<(), CompanionPreflightStop> {
    for key in keys.keys() {
        detach_key(state, id, key)?;
    }

    state.posting_bindings.remove(&id);
    Ok(())
}

/// Deferred read cleanup owns only the touched bucket, never its old footprint.
pub(in super::super) fn remove_physical_key(
    state: &mut MarkState,
    id: u64,
    key: &FactPostingKey,
) -> Result<(), CompanionPreflightStop> {
    detach_key(state, id, key)?;
    Ok(())
}

fn detach_key(
    state: &mut MarkState,
    id: u64,
    key: &FactPostingKey,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();

    let Some((stored_key, groups)) = state.postings.get_key_value(key) else {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    };
    let stored_key = Arc::clone(stored_key);
    let mut groups = groups.clone();

    if groups.remove(&id).is_none() {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    }
    debit(&mut state.posting_count, 1)?;
    if groups.is_empty() {
        state.postings.remove(key);
    } else {
        state.postings.insert(stored_key, groups);
    }
    Ok(())
}
