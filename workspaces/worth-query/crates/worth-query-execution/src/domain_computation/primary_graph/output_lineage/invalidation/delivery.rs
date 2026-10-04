use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::mvcc::{
    CompanionPreflightBudget, CompanionPreflightStop, PublicationCompanionPreflight,
};

use crate::domain_computation::execution_runtime::source_invalidation::{
    RetainedInvalidationCapacity, WorthQueryInvalidationResources,
};
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::logical_marking::{
    AppliedNativeMarking, LogicalMarkingCounts, NativeMarkingPrecision, NativeMarkingReport,
};
use super::mark_state::{DeliveryEpoch, FullVerificationReason, MarkState, SettlementMarks};
use super::{fact_key::FactPostingKey, touch_keys};
use super::{index_capacity, retention};

mod meter;
use meter::MarkingMeter;
pub(super) use meter::PrepaidAdmission;

/// Materialize only the native changes delivered by this publication. It is
/// retained for late settlement insertion, never used as a substitute fact.
///
/// Extraction is linear in the commit the writer has already admitted, so it
/// is charged as preparation bytes and interruption checkpoints only. A commit
/// whose selectors exceed preparation memory is delivered as a declared-change
/// discontinuity rather than refusing the writer.
pub(super) fn selectors(
    context: &mut PublicationCompanionPreflight<'_>,
    budget: CompanionPreflightBudget,
) -> Result<(Option<Arc<[FactPostingKey]>>, u64), CompanionPreflightStop> {
    let mut meter = MarkingMeter::new(context, budget);
    let extracted = extract(context, &mut meter);
    let extraction_bytes = meter.charged_bytes();
    match extracted.and_then(|kept| context.bytes(extraction_bytes).map(|()| kept)) {
        Ok(kept) => Ok(kept),
        Err(stop) if meter::degrades_delivery(&stop) => Ok((None, 0)),
        Err(stop) => Err(stop),
    }
}

fn extract(
    context: &PublicationCompanionPreflight<'_>,
    meter: &mut MarkingMeter<'_, '_>,
) -> Result<(Option<Arc<[FactPostingKey]>>, u64), CompanionPreflightStop> {
    let overflow = CompanionPreflightStop::PreparationMemoryCounterOverflow;
    let Some(touches) = context
        .canonical_commit()
        .descriptive_touches()
        .exact_touches()
    else {
        return Ok((None, 0));
    };
    let capacity = touches.len().checked_mul(2).ok_or(overflow)?;
    meter.bytes(
        u64::try_from(capacity)
            .ok()
            .and_then(|n| n.checked_mul(std::mem::size_of::<FactPostingKey>() as u64))
            .ok_or(overflow)?,
    )?;
    let mut keys = Vec::with_capacity(capacity);
    let mut retained_payload = 0u64;
    for touch in touches {
        meter.checkpoint()?;
        let payload_bytes = touch_keys::payload_capacity(touch).ok_or(overflow)?;
        meter.bytes(payload_bytes)?;
        retained_payload = retained_payload
            .checked_add(payload_bytes)
            .ok_or(overflow)?;
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
        .ok_or(overflow)?;
    meter.bytes(retained_bytes)?;
    Ok((
        Some(keys.into()),
        retained_bytes
            .checked_add(retained_payload)
            .ok_or(overflow)?,
    ))
}

/// The marked branch state, its delivery report and the required-work hints
/// selected by that marking, with their preparation already admitted.
pub(super) struct MarkedDelivery {
    pub(super) state: MarkState,
    pub(super) report: NativeMarkingReport,
    pub(super) selected: Vec<SelectedHint>,
    pub(super) hint_allowance: PrepaidAdmission,
    /// Retained for late settlement replay; absent after a discontinuity.
    pub(super) keys: Option<Arc<[FactPostingKey]>>,
    pub(super) retained_key_bytes: u64,
}

pub(super) type SelectedHint = Arc<RequiredWorkMembership>;

/// Marking either completes within the installed marking ceiling and retained
/// capacity or degrades to a counted discontinuity: readers then fall back to
/// full verification. Reader fan-out never refuses or livelocks a legal writer.
pub(super) fn mark(
    observed: &MarkState,
    (keys, retained_key_bytes): (Option<Arc<[FactPostingKey]>>, u64),
    commit: CommitId,
    (budget, resources): (CompanionPreflightBudget, &WorthQueryInvalidationResources),
    context: &mut PublicationCompanionPreflight<'_>,
) -> Result<MarkedDelivery, CompanionPreflightStop> {
    // One live clone at a time: a degraded delivery drops the marked clone
    // before taking the unmarked one.
    context.bytes(
        index_capacity::arc_bytes::<MarkState>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    let branch_bytes = context.branch_id().0.len();
    let mut counts = LogicalMarkingCounts::default();
    let mut meter = MarkingMeter::new(context, budget);
    let mut state = observed.clone();
    let attempt =
        apply(&mut state, keys.as_deref(), commit, &mut meter, &mut counts).and_then(|applied| {
            let (selected, hint_allowance) =
                select_hints(&state, &applied.affected, branch_bytes, &mut meter)?;
            Ok((applied.report, selected, hint_allowance))
        });
    let marking_bytes = meter.charged_bytes();
    let kept = match attempt.and_then(|kept| context.bytes(marking_bytes).map(|()| kept)) {
        Ok(kept) => kept,
        Err(stop) if meter::degrades_delivery(&stop) => {
            drop(state);
            let precision = NativeMarkingPrecision::MarkingCeilingExceeded(counts);
            return discontinuity(observed, commit, precision, resources, context);
        }
        Err(stop) => return Err(stop),
    };
    match retention::admit_version(&mut state, marking_bytes, resources, context) {
        Ok(()) => {}
        Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => {
            drop((state, kept));
            let precision = NativeMarkingPrecision::RetainedCapacityExhausted(counts);
            return discontinuity(observed, commit, precision, resources, context);
        }
        Err(stop) => return Err(stop),
    }
    let (report, selected, hint_allowance) = kept;
    Ok(MarkedDelivery {
        state,
        report,
        selected,
        hint_allowance,
        keys,
        retained_key_bytes,
    })
}

/// The unmarked prior state publishes under a new discontinuity epoch. Its
/// version copies no node; a refusal of the version itself is not degradable.
fn discontinuity(
    observed: &MarkState,
    commit: CommitId,
    precision: NativeMarkingPrecision,
    resources: &WorthQueryInvalidationResources,
    context: &mut PublicationCompanionPreflight<'_>,
) -> Result<MarkedDelivery, CompanionPreflightStop> {
    let mut state = observed.clone();
    state.delivery_epoch = DeliveryEpoch::after_discontinuity(commit);
    state.last_discontinuity = Some(FullVerificationReason::DeclaredChangeUnavailable);
    retention::admit_version(&mut state, 0, resources, context)?;
    Ok(MarkedDelivery {
        state,
        report: NativeMarkingReport { commit, precision },
        selected: Vec::new(),
        hint_allowance: PrepaidAdmission::default(),
        keys: None,
        retained_key_bytes: 0,
    })
}

/// Selection and each selected hint's preparation are proportional to the
/// marked closure, so they are admitted with it.
fn select_hints(
    state: &MarkState,
    affected: &OrdSet<Arc<RecordedSettlementIdentity>>,
    branch_bytes: usize,
    meter: &mut MarkingMeter<'_, '_>,
) -> Result<(Vec<SelectedHint>, PrepaidAdmission), CompanionPreflightStop> {
    let overflow = CompanionPreflightStop::PreparationMemoryCounterOverflow;
    meter.bytes(
        u64::try_from(affected.len())
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<SelectedHint>() as u64))
            .ok_or(overflow)?,
    )?;
    let mut selected = Vec::with_capacity(affected.len());
    for identity in affected {
        meter.work(2)?;
        meter.ordered_read(state.settlements.len())?;
        if let Some(membership) = state
            .settlements
            .get(identity)
            .and_then(|row| row.work_membership.as_ref())
        {
            selected.push(Arc::clone(membership));
        }
    }
    if selected.is_empty() {
        return Ok((selected, PrepaidAdmission::default()));
    }
    // Per hint: its prepared slot, retained hint and branch copy, two retained
    // capacity tickets, and the branch copy plus fixed construction work.
    let ticket = index_capacity::arc_bytes::<RetainedInvalidationCapacity>().ok_or(overflow)?;
    let per_hint_bytes = RequiredWorkMembership::native_branch_retained_bytes(branch_bytes)
        .and_then(|bytes| bytes.checked_add(RequiredWorkMembership::native_hint_bytes()))
        .and_then(|bytes| bytes.checked_add(ticket.checked_mul(2)?))
        .and_then(|bytes| bytes.checked_add((2 * std::mem::size_of::<usize>()) as u64))
        .ok_or(overflow)?;
    let per_hint_work = u64::try_from(branch_bytes)
        .ok()
        .and_then(|bytes| bytes.checked_add(7))
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    let count = u64::try_from(selected.len()).map_err(|_| overflow)?;
    let hint_bytes = per_hint_bytes.checked_mul(count).ok_or(overflow)?;
    let hint_work = per_hint_work
        .checked_mul(count)
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    meter.bytes(hint_bytes)?;
    meter.work(hint_work)?;
    Ok((selected, PrepaidAdmission::new(hint_work, hint_bytes)))
}

/// Touch matching visits only selected posting buckets. Each affected ordinal
/// is marked once; downstream propagation follows only actual consumed edges.
fn apply(
    state: &mut MarkState,
    keys: Option<&[FactPostingKey]>,
    commit: CommitId,
    meter: &mut MarkingMeter<'_, '_>,
    counts: &mut LogicalMarkingCounts,
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
    let mut affected = OrdSet::new();
    for key in keys {
        // Probing the admitted commit's own keys is linear in that commit.
        // Marking work is spent only on matched postings and their closure.
        meter.checkpoint()?;
        counts.posting_key_lookups = counts
            .posting_key_lookups
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        let Some(postings) = state.postings.get(key) else {
            continue;
        };
        for posting in postings {
            meter.work(1)?;
            counts.matched_fact_postings = counts
                .matched_fact_postings
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            meter.ordered_read(state.settlements.len())?;
            let Some(existing) = state.settlements.get(&posting.settlement) else {
                continue;
            };
            meter.ordered_read(existing.dirty_ordinals.len())?;
            if existing.dirty_ordinals.contains(&posting.ordinal) {
                continue;
            }
            meter.bytes(
                index_capacity::arc_bytes::<SettlementMarks>()
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            meter.ordered_edit::<usize, ()>(existing.dirty_ordinals.len())?;
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
            meter.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
                state.settlements.len(),
            )?;
            state
                .settlements
                .insert(Arc::clone(&posting.settlement), Arc::new(row));
            meter.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(affected.len())?;
            affected.insert(Arc::clone(&posting.settlement));
        }
    }
    let affected = propagate(state, affected, meter, counts)?;
    counts.visited_vertices =
        u64::try_from(affected.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    Ok(AppliedNativeMarking {
        affected,
        report: NativeMarkingReport {
            commit,
            precision: NativeMarkingPrecision::Exact(*counts),
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
