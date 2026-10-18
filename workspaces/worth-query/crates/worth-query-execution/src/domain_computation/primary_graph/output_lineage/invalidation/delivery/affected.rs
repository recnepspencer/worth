//! Native touched-key matching and propagation through consumed output edges.
use std::sync::Arc;

use super::{
    index_capacity, AppliedNativeMarking, CommitId, CompanionPreflightStop, DeliveryEpoch,
    FactPostingKey, FullVerificationReason, MarkState, MarkingMeter, NativeMarkingReport,
    RecordedSettlementIdentity, RetainedIndexAdmission, SettlementMarks,
    WorthQueryNativeMarkingPrecision, WorthQueryNativeMarkingWork,
};
use im::OrdSet;
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) mod posting_order;

/// Touch matching visits only selected posting buckets. Each affected ordinal
/// is marked once; downstream propagation follows only actual consumed edges.
pub(super) fn apply(
    state: &mut MarkState,
    keys: Option<&[FactPostingKey]>,
    commit: CommitId,
    meter: &mut MarkingMeter<'_, '_>,
    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<AppliedNativeMarking, CompanionPreflightStop> {
    let Some(keys) = keys else {
        state.delivery_epoch = DeliveryEpoch::after_discontinuity(commit);
        state.last_discontinuity = Some(FullVerificationReason::DeclaredChangeUnavailable);
        // The installed image carries this epoch. Its required-only registry
        // cursor schedules exceptional re-evidence without visiting unrelated
        // historical actor rows during native publication.
        return Ok(AppliedNativeMarking {
            affected: OrdSet::new(),
            report: NativeMarkingReport {
                commit,
                precision: WorthQueryNativeMarkingPrecision::DeclaredChangeUnavailable,
            },
        });
    };
    let affected = mark_keys(state, keys, meter, counts)?;
    let affected = propagate(state, affected, meter, counts)?;
    counts.visited_vertices =
        u64::try_from(affected.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    Ok(AppliedNativeMarking {
        affected,
        report: NativeMarkingReport {
            commit,
            precision: WorthQueryNativeMarkingPrecision::Exact(*counts),
        },
    })
}

/// The same native fact-ordinal marking for physical source publication and
/// owner-issued publication completion. No second consumer registry is used.
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn mark_keys(
    state: &mut MarkState,
    keys: &[FactPostingKey],
    meter: &mut impl RetainedIndexAdmission,
    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let mut affected = OrdSet::new();
    for key in keys {
        // Probing the admitted commit's own keys is linear in that commit.
        // Marking work is spent only on matched postings and their closure.
        meter.work(0)?;
        counts.posting_key_lookups = counts
            .posting_key_lookups
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        let Some(mut postings) =
            posting_order::LogicalPostings::prepare(state, key, meter, counts)?
        else {
            continue;
        };
        while let Some(posting) = postings.next(state, key, meter, counts)? {
            meter.work(1)?;
            meter.ordered_read(state.settlements.len())?;
            let Some(existing) = state.settlements.get(&posting.settlement) else {
                continue;
            };
            meter.ordered_read(existing.dirty_ordinals.len())?;
            if existing.dirty_ordinals.contains(&posting.ordinal) {
                continue;
            }
            meter.index_bytes(
                index_capacity::arc_bytes::<SettlementMarks>()
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            meter.index_edit::<usize, ()>(existing.dirty_ordinals.len())?;
            let mut row = (**existing).clone();
            row.dirty_ordinals.insert(posting.ordinal);
            if let Some(frontier) = &mut row.computation_frontier {
                meter.ordered_read(frontier.len())?;
                if !frontier.contains(&posting.ordinal) {
                    meter.index_edit::<usize, ()>(frontier.len())?;
                    frontier.insert(posting.ordinal);
                    state.computation_dirty_ordinal_count = state
                        .computation_dirty_ordinal_count
                        .checked_add(1)
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                }
            }
            counts.marked_fact_ordinals = counts
                .marked_fact_ordinals
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            state.dirty_ordinal_count = state
                .dirty_ordinal_count
                .checked_add(1)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            meter.index_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
                state.settlements.len(),
            )?;
            state
                .settlements
                .insert(Arc::clone(&posting.settlement), Arc::new(row));
            meter.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(affected.len())?;
            affected.insert(Arc::clone(&posting.settlement));
        }
    }
    Ok(affected)
}

pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn propagate(
    state: &mut MarkState,
    mut waiting: OrdSet<Arc<RecordedSettlementIdentity>>,
    admission: &mut impl RetainedIndexAdmission,
    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let mut visited = OrdSet::new();
    while let Some(upstream) = waiting.get_min().cloned() {
        admission.work(1)?;
        admission.ordered_remove::<Arc<RecordedSettlementIdentity>, ()>(waiting.len())?;
        waiting.remove(&upstream);
        admission.ordered_read(visited.len())?;
        if visited.contains(&upstream) {
            continue;
        }
        // The walk creates `visited` empty and never shares it while it grows.
        admission.fresh_insert::<Arc<RecordedSettlementIdentity>, ()>(visited.len())?;
        visited.insert(Arc::clone(&upstream));
        // A native change to an alias can change the output consumed through
        // any earlier exact identity in its certified equality chain.
        admission.ordered_read(state.equal_links.len())?;
        if let Some(prior) = state
            .equal_links
            .get(&upstream)
            .and_then(|link| link.prior.as_ref())
        {
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(waiting.len())?;
            waiting.insert(Arc::clone(prior));
        }
        admission.ordered_read(state.downstream.len())?;
        let Some(downstream) = state.downstream.get(&upstream).cloned() else {
            continue;
        };
        for target in downstream {
            admission.work(1)?;
            counts.downstream_edges = counts
                .downstream_edges
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            admission.ordered_read(state.settlements.len())?;
            let Some(existing) = state.settlements.get(&target) else {
                continue;
            };
            admission.ordered_read(existing.pending_upstream.len())?;
            if !existing.pending_upstream.contains(&upstream) {
                admission.index_bytes(
                    index_capacity::arc_bytes::<SettlementMarks>()
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                )?;
                admission.index_edit::<Arc<RecordedSettlementIdentity>, ()>(
                    existing.pending_upstream.len(),
                )?;
                let mut row = (**existing).clone();
                row.pending_upstream.insert(Arc::clone(&upstream));
                state.pending_edge_count = state
                    .pending_edge_count
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                admission.index_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
                    state.settlements.len(),
                )?;
                state.settlements.insert(Arc::clone(&target), Arc::new(row));
            }
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(waiting.len())?;
            waiting.insert(target);
        }
    }
    Ok(visited)
}
