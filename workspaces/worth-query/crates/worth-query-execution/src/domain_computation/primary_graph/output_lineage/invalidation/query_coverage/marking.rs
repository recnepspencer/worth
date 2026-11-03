//! Query reads use native touched buckets alongside settlement consumers.
use super::super::logical_marking::WorthQueryNativeMarkingWork;
use super::super::mark_state::MarkState;
use super::super::settlement;
use super::super::settlement::PostingBindings;
use super::*;

pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn mark_query_reads(
    state: &mut MarkState,
    key: &FactPostingKey,

    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<(), CompanionPreflightStop> {
    let Some(groups) = state.postings.get(key).cloned() else {
        return Ok(());
    };
    for id in groups {
        let Some(binding) = state.posting_bindings.get(&id).cloned() else {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        };
        let PostingBindings::QueryRead(read) = binding.as_ref() else {
            continue;
        };

        let ordinals = read
            .projection
            .keys()
            .get(key)
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;

        let ordinal = *ordinals
            .get_min()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        counts.matched_fact_postings = counts
            .matched_fact_postings
            .checked_add(ordinals.len() as u64)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        if read.identity.strong_count() == 0 {
            let remaining = read.projection.detach(key)?;
            settlement::remove_physical_key(state, id, key)?;
            if let Some(projection) = remaining {
                let mut replacement = read.clone();
                replacement.projection = projection;
                state
                    .posting_bindings
                    .insert(id, Arc::new(PostingBindings::QueryRead(replacement)));
            } else {
                state.posting_bindings.remove(&id);
            }
            continue;
        }
        if read.dirty.is_some() {
            continue;
        }

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
