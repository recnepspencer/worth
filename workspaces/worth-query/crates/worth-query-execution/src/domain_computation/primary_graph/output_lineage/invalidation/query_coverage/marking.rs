//! Query reads use native touched buckets alongside settlement consumers.
use super::super::{
    admission::RetainedIndexAdmission,
    logical_marking::WorthQueryNativeMarkingWork,
    mark_state::MarkState,
    settlement::{self, PostingBindings},
};
use super::*;

pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn mark_query_reads(
    state: &mut MarkState,
    key: &FactPostingKey,
    admission: &mut impl RetainedIndexAdmission,
    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<(), CompanionPreflightStop> {
    admission.key_read(key, state.postings.len())?;
    let Some(groups) = state.postings.get(key).cloned() else {
        return Ok(());
    };
    for id in groups {
        admission.work(1)?;
        admission.ordered_read(state.posting_bindings.len())?;
        let Some(binding) = state.posting_bindings.get(&id).cloned() else {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        };
        let PostingBindings::QueryRead(read) = binding.as_ref() else {
            continue;
        };
        admission.key_read(key, read.projection.keys().len())?;
        let ordinals = read
            .projection
            .keys()
            .get(key)
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        admission.ordered_read(ordinals.len())?;
        let ordinal = *ordinals
            .get_min()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        counts.matched_fact_postings = counts
            .matched_fact_postings
            .checked_add(ordinals.len() as u64)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        if read.identity.strong_count() == 0 {
            let remaining = read.projection.detach(key, admission)?;
            settlement::remove_physical_key(state, id, key, admission)?;
            if let Some(projection) = remaining {
                admission.index_bytes(
                    index_capacity::arc_bytes::<PostingBindings>()
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                )?;
                admission.index_edit::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
                let mut replacement = read.clone();
                replacement.projection = projection;
                state
                    .posting_bindings
                    .insert(id, Arc::new(PostingBindings::QueryRead(replacement)));
            } else {
                admission
                    .index_remove::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
                state.posting_bindings.remove(&id);
            }
            continue;
        }
        if read.dirty.is_some() {
            continue;
        }
        admission.index_bytes(
            index_capacity::arc_bytes::<PostingBindings>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.index_edit::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
        let mut replacement = read.clone();
        replacement.dirty = Some(ordinal); // One actual matched original ordinal suffices to require full verification.
        state
            .posting_bindings
            .insert(id, Arc::new(PostingBindings::QueryRead(replacement)));
        counts.marked_fact_ordinals = counts
            .marked_fact_ordinals
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    }
    Ok(())
}
