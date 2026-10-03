use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::mvcc::{CompanionPreflightStop, PublicationCompanionPreflight};

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::index_capacity;
use super::logical_marking::{
    AppliedNativeMarking, LogicalMarkingCounts, NativeMarkingPrecision, NativeMarkingReport,
};
use super::mark_state::{DeliveryEpoch, FullVerificationReason, MarkState, SettlementMarks};
use super::{fact_key::FactPostingKey, touch_keys};

/// Materialize only the native changes delivered by this publication. It is
/// retained for late settlement insertion, never used as a substitute fact.
///
/// Extraction is linear in the commit the writer has already admitted, so it
/// is charged as preparation bytes and interruption checkpoints only. The
/// marking work ceiling bounds index matching and downstream fan-out, not the
/// size of a legal commit.
pub(super) fn selectors(
    context: &mut PublicationCompanionPreflight<'_>,
) -> Result<(Option<Arc<[FactPostingKey]>>, u64), CompanionPreflightStop> {
    let Some(touches) = context
        .canonical_commit()
        .descriptive_touches()
        .exact_touches()
    else {
        return Ok((None, 0));
    };
    let touch_count = touches.len();
    let capacity = touch_count
        .checked_mul(2)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    context.bytes(
        u64::try_from(capacity)
            .ok()
            .and_then(|n| n.checked_mul(std::mem::size_of::<FactPostingKey>() as u64))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    let mut keys = Vec::with_capacity(capacity);
    let mut retained_payload = 0u64;
    for ordinal in 0..touch_count {
        context.checkpoint()?;
        let touch = &context
            .canonical_commit()
            .descriptive_touches()
            .exact_touches()
            .expect("precision is sealed")[ordinal];
        let payload_bytes = touch_keys::payload_capacity(touch)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        context.bytes(payload_bytes)?;
        retained_payload = retained_payload
            .checked_add(payload_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let touch = &context
            .canonical_commit()
            .descriptive_touches()
            .exact_touches()
            .expect("precision is sealed")[ordinal];
        touch_keys::visit(
            touch,
            |_| Ok(()),
            |key| {
                keys.push(key);
                Ok(())
            },
        )?;
    }
    // Vec -> Arc temporarily owns both allocations.
    let retained_bytes = (keys.len() as u64)
        .checked_mul(std::mem::size_of::<FactPostingKey>() as u64)
        .and_then(|n| n.checked_add(2 * std::mem::size_of::<usize>() as u64))
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    context.bytes(retained_bytes)?;
    Ok((
        Some(keys.into()),
        retained_bytes
            .checked_add(retained_payload)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    ))
}

/// Touch matching visits only selected posting buckets. Each affected ordinal
/// is marked once; downstream propagation follows only actual consumed edges.
pub(super) fn apply(
    state: &mut MarkState,
    keys: Option<&[FactPostingKey]>,
    commit: worth_relational::facade::history::CommitId,
    admission: &mut impl IndexAdmission,
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
                precision: NativeMarkingPrecision::DeclaredChangeUnavailable,
            },
        });
    };
    let mut counts = LogicalMarkingCounts::default();
    let mut affected = OrdSet::new();
    for key in keys {
        admission.work(1)?;
        admission.key_read(key, state.postings.len())?;
        counts.posting_key_lookups = counts
            .posting_key_lookups
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        let Some(postings) = state.postings.get(key) else {
            continue;
        };
        for posting in postings {
            admission.work(1)?;
            counts.matched_fact_postings = counts
                .matched_fact_postings
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            admission.ordered_read(state.settlements.len())?;
            let Some(existing) = state.settlements.get(&posting.settlement) else {
                continue;
            };
            admission.ordered_read(existing.dirty_ordinals.len())?;
            if existing.dirty_ordinals.contains(&posting.ordinal) {
                continue;
            }
            admission.bytes(
                index_capacity::arc_bytes::<SettlementMarks>()
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            admission.ordered_edit::<usize, ()>(existing.dirty_ordinals.len())?;
            let mut row = (**existing).clone();
            row.dirty_ordinals.insert(posting.ordinal);
            counts.marked_fact_ordinals = counts
                .marked_fact_ordinals
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            state.dirty_ordinal_count = state
                .dirty_ordinal_count
                .checked_add(1)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
                state.settlements.len(),
            )?;
            state
                .settlements
                .insert(Arc::clone(&posting.settlement), Arc::new(row));
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(affected.len())?;
            affected.insert(Arc::clone(&posting.settlement));
        }
    }
    let affected = propagate(state, affected, admission, &mut counts)?;
    counts.visited_vertices =
        u64::try_from(affected.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    Ok(AppliedNativeMarking {
        affected,
        report: NativeMarkingReport {
            commit,
            precision: NativeMarkingPrecision::Exact(counts),
        },
    })
}

pub(super) fn propagate(
    state: &mut MarkState,
    mut waiting: OrdSet<Arc<RecordedSettlementIdentity>>,
    admission: &mut impl IndexAdmission,
    counts: &mut LogicalMarkingCounts,
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
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(visited.len())?;
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
                admission.bytes(
                    index_capacity::arc_bytes::<SettlementMarks>()
                        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
                )?;
                admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(
                    existing.pending_upstream.len(),
                )?;
                let mut row = (**existing).clone();
                row.pending_upstream.insert(Arc::clone(&upstream));
                state.pending_edge_count = state
                    .pending_edge_count
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
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
