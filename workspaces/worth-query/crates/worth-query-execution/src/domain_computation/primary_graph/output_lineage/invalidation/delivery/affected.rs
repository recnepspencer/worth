//! Native touched-key matching and propagation through consumed output edges.
use std::sync::Arc;

use super::AppliedNativeMarking;
use super::CommitId;
use super::CompanionPreflightStop;
use super::DeliveryEpoch;
use super::FactPostingKey;
use super::FullVerificationReason;
use super::MarkState;
use super::NativeMarkingReport;
use super::RecordedSettlementIdentity;
use super::WorthQueryNativeMarkingPrecision;
use super::WorthQueryNativeMarkingWork;
use im::OrdSet;
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) mod posting_order;

/// Touch matching visits only selected posting buckets. Each affected ordinal
/// is marked once; downstream propagation follows only actual consumed edges.
pub(super) fn apply(
    state: &mut MarkState,
    keys: Option<&[FactPostingKey]>,
    commit: CommitId,

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
    let affected = mark_keys(state, keys, counts)?;
    let affected = propagate(state, affected, counts)?;
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

    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let mut affected = OrdSet::new();
    for key in keys {
        // Probing the admitted commit's own keys is linear in that commit.
        // Marking work is spent only on matched postings and their closure.

        counts.posting_key_lookups = counts
            .posting_key_lookups
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        super::super::query_coverage::mark_query_reads(state, key, counts)?;
        let Some(mut postings) = posting_order::LogicalPostings::prepare(state, key, counts)?
        else {
            continue;
        };
        while let Some(posting) = postings.next(state, key, counts)? {
            let Some(existing) = state.settlements.get(&posting.settlement) else {
                continue;
            };

            if existing.dirty_ordinals.contains(&posting.ordinal) {
                continue;
            }

            let mut row = (**existing).clone();
            row.dirty_ordinals.insert(posting.ordinal);
            if let Some(frontier) = &mut row.computation_frontier
                && !frontier.contains(&posting.ordinal)
            {
                frontier.insert(posting.ordinal);
                state.computation_dirty_ordinal_count = state
                    .computation_dirty_ordinal_count
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            }
            counts.marked_fact_ordinals = counts
                .marked_fact_ordinals
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            state.dirty_ordinal_count = state
                .dirty_ordinal_count
                .checked_add(1)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;

            state
                .settlements
                .insert(Arc::clone(&posting.settlement), Arc::new(row));

            affected.insert(Arc::clone(&posting.settlement));
        }
    }
    Ok(affected)
}

pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn propagate(
    state: &mut MarkState,
    mut waiting: OrdSet<Arc<RecordedSettlementIdentity>>,

    counts: &mut WorthQueryNativeMarkingWork,
) -> Result<OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
    let mut visited = OrdSet::new();
    while let Some(upstream) = waiting.get_min().cloned() {
        waiting.remove(&upstream);

        if visited.contains(&upstream) {
            continue;
        }
        // The walk creates `visited` empty and never shares it while it grows.

        visited.insert(Arc::clone(&upstream));
        // A native change to an alias can change the output consumed through
        // any earlier exact identity in its certified equality chain.

        if let Some(prior) = state
            .equal_links
            .get(&upstream)
            .and_then(|link| link.prior.as_ref())
        {
            waiting.insert(Arc::clone(prior));
        }

        let Some(downstream) = state.downstream.get(&upstream).cloned() else {
            continue;
        };
        for target in downstream {
            counts.downstream_edges = counts
                .downstream_edges
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;

            let Some(existing) = state.settlements.get(&target) else {
                continue;
            };

            if !existing.pending_upstream.contains(&upstream) {
                let mut row = (**existing).clone();
                row.pending_upstream.insert(Arc::clone(&upstream));
                state.pending_edge_count = state
                    .pending_edge_count
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;

                state.settlements.insert(Arc::clone(&target), Arc::new(row));
            }

            waiting.insert(target);
        }
    }
    Ok(visited)
}
