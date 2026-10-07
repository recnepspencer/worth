use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::delivery;
use super::fact_key::FactPostingKey;
use super::index_capacity;
use super::logical_marking::LogicalMarkingCounts;
use super::mark_state::{
    FactPosting, FullVerificationReason, MarkState, OutputFactCoverage, SettlementMarks,
};
use super::output_facts::RegisteredOutputFacts;
use super::source_alignment::BranchMarkRoot;
use super::InvalidationEditAdmission;

mod postings;

pub(super) enum SettlementReadAlignment {
    Current,
    Retained,
}

pub(in crate::domain_computation::primary_graph) struct SettlementRegistration {
    pub(in crate::domain_computation::primary_graph) work_membership:
        Option<Arc<RequiredWorkMembership>>,
    pub(in crate::domain_computation::primary_graph) identity: Arc<RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) facts: super::super::RetainedSourceFacts,
    pub(in crate::domain_computation::primary_graph) output_facts: Option<RegisteredOutputFacts>,
    pub(in crate::domain_computation::primary_graph) read_basis: PositionedRelationalSnapshot,
    /// Fact ordinals the registrant already knows are not current at
    /// `read_basis`: no delivery after that basis would ever mark them.
    pub(in crate::domain_computation::primary_graph) stale_at_read_basis: OrdSet<usize>,
    pub(in crate::domain_computation::primary_graph) requirement: Option<FullVerificationReason>,
    pub(in crate::domain_computation::primary_graph) upstream:
        OrdSet<Arc<RecordedSettlementIdentity>>,
}

pub(super) struct AdmittedSettlementRegistration {
    pub(super) input: SettlementRegistration,
    pub(super) fact_capacity: Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>,
}

/// Install exact consumed facts into the selected live reverse index. The
/// original facts remain in one Arc; posting buckets retain ordinal references.
pub(super) fn insert(
    state: &mut MarkState,
    registration: AdmittedSettlementRegistration,
    root: &BranchMarkRoot,
    alignment: SettlementReadAlignment,
    output_coverage: OutputFactCoverage,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let AdmittedSettlementRegistration {
        input,
        fact_capacity,
    } = registration;
    let SettlementRegistration {
        work_membership,
        identity,
        facts,
        output_facts,
        read_basis,
        stale_at_read_basis,
        requirement,
        upstream,
    } = input;
    let identity = &identity;
    remove(state, identity, admission)?;
    let mut verification_requirement = requirement;
    let prepared = postings::PreparedPostingOrdinals::prepare(
        facts.postconditions(),
        output_facts.as_ref().map(|set| set.facts.as_ref()),
        &mut verification_requirement,
        admission,
    )?;
    let (ordinals, posting_payload_bytes) = prepared.install(state, identity, admission)?;
    let mut row = SettlementMarks {
        work_membership,
        _fact_capacity: fact_capacity,
        posting_payload_bytes,
        posting_ordinals: ordinals,
        consumed_upstream: upstream,
        facts,
        output_facts,
        output_coverage,
        read_basis: Arc::new(read_basis),
        delivery_epoch: state.delivery_epoch,
        dirty_ordinals: stale_at_read_basis,
        pending_upstream: OrdSet::new(),
        verification_requirement,
        superseded: false,
    };
    let replayed_stale = matches!(alignment, SettlementReadAlignment::Retained)
        && replay(&mut row, root, admission)?;
    state.settlement_key_payload_bytes = state
        .settlement_key_payload_bytes
        .checked_add(posting_payload_bytes)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    for upstream in &row.consumed_upstream {
        admission.work(1)?;
        admission.ordered_read(state.downstream.len())?;
        let mut targets = state.downstream.get(upstream).cloned().unwrap_or_default();
        admission.ordered_read(targets.len())?;
        if !targets.contains(identity) {
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(targets.len())?;
            targets.insert(Arc::clone(identity));
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, OrdSet<Arc<RecordedSettlementIdentity>>>(state.downstream.len())?;
            state.downstream.insert(Arc::clone(upstream), targets);
            state.downstream_edge_count = state
                .downstream_edge_count
                .checked_add(1)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }
        admission.ordered_read(state.settlements.len())?;
        if state.settlements.get(upstream).is_none_or(|upstream| {
            upstream.delivery_epoch != state.delivery_epoch
                || upstream.verification_requirement.is_some()
                || !upstream.dirty_ordinals.is_empty()
                || !upstream.pending_upstream.is_empty()
        }) {
            admission
                .ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(row.pending_upstream.len())?;
            row.pending_upstream.insert(Arc::clone(upstream));
        }
    }
    state.dirty_ordinal_count = state
        .dirty_ordinal_count
        .checked_add(row.dirty_ordinals.len())
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    state.pending_edge_count = state
        .pending_edge_count
        .checked_add(row.pending_upstream.len())
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.bytes(
        index_capacity::arc_bytes::<SettlementMarks>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
        state.settlements.len(),
    )?;
    // Dirty, pending, epoch-stale or verification-required rows are not clean.
    let not_clean = replayed_stale
        || row.delivery_epoch != state.delivery_epoch
        || row.verification_requirement.is_some()
        || !row.dirty_ordinals.is_empty()
        || !row.pending_upstream.is_empty();
    state
        .settlements
        .insert(Arc::clone(identity), Arc::new(row));
    if not_clean {
        // Consumers registered against the earlier row must not stay clean
        // behind a replacement that is not itself clean.
        delivery::propagate(
            state,
            OrdSet::unit(Arc::clone(identity)),
            admission,
            &mut LogicalMarkingCounts::default(),
        )?;
    }
    Ok(())
}

/// Replays retained deliveries after the read basis and reports whether the
/// row is no longer current.
fn replay(
    row: &mut SettlementMarks,
    root: &BranchMarkRoot,
    admission: &mut impl IndexAdmission,
) -> Result<bool, CompanionPreflightStop> {
    admission.ordered_read(root.past.len())?;
    let Some(basis) = root.past.get(&row.read_basis.position()).filter(|basis| {
        basis.root_id == row.read_basis.root_id() && basis.commit_id == row.read_basis.commit_id()
    }) else {
        // The caller omits replay only when its basis is the live cell image.
        row.verification_requirement = Some(FullVerificationReason::RetainedDeliveryGap);
        return Ok(true);
    };
    row.delivery_epoch = basis.state.delivery_epoch;
    for (_, historical) in root.past.range(row.read_basis.position()..) {
        admission.work(1)?;
        let delivery = &historical.next_delivery;
        let Some(keys) = &delivery.keys else {
            row.verification_requirement = Some(FullVerificationReason::DeclaredChangeUnavailable);
            continue;
        };
        for key in keys.iter() {
            admission.work(1)?;
            admission.key_read(key, row.posting_ordinals.len())?;
            let Some(ordinals) = row.posting_ordinals.get(key) else {
                continue;
            };
            for ordinal in ordinals {
                admission.work(1)?;
                admission.ordered_edit::<usize, ()>(row.dirty_ordinals.len())?;
                row.dirty_ordinals.insert(*ordinal);
            }
        }
    }
    Ok(!row.dirty_ordinals.is_empty() || row.verification_requirement.is_some())
}

pub(super) fn remove(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    admission.ordered_read(state.settlements.len())?;
    let Some(row) = state.settlements.get(identity).cloned() else {
        return Ok(());
    };
    for (key, ordinals) in &row.posting_ordinals {
        admission.work(1)?;
        admission.key_read(key, state.postings.len())?;
        let Some(mut postings) = state.postings.get(key).cloned() else {
            continue;
        };
        for ordinal in ordinals {
            let posting = FactPosting {
                settlement: Arc::clone(identity),
                ordinal: *ordinal,
            };
            admission.work(1)?;
            admission.ordered_remove::<FactPosting, ()>(postings.len())?;
            if postings.remove(&posting).is_some() {
                debit(&mut state.posting_count, 1)?;
            }
        }
        if postings.is_empty() {
            admission.key_remove::<Arc<FactPostingKey>, OrdSet<FactPosting>>(
                key,
                state.postings.len(),
            )?;
            state.postings.remove(key);
            let key_bytes = key
                .owned_payload_capacity_bytes()
                .and_then(|n| n.checked_add(index_capacity::arc_bytes::<FactPostingKey>()?))
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            debit(&mut state.key_payload_bytes, key_bytes)?;
        } else {
            admission
                .key_edit::<Arc<FactPostingKey>, OrdSet<FactPosting>>(key, state.postings.len())?;
            state.postings.insert(Arc::clone(key), postings);
        }
    }
    for upstream in &row.consumed_upstream {
        admission.work(1)?;
        admission.ordered_read(state.downstream.len())?;
        let Some(mut targets) = state.downstream.get(upstream).cloned() else {
            continue;
        };
        admission.ordered_remove::<Arc<RecordedSettlementIdentity>, ()>(targets.len())?;
        if targets.remove(identity).is_some() {
            debit(&mut state.downstream_edge_count, 1)?;
        }
        if targets.is_empty() {
            admission.ordered_remove::<Arc<RecordedSettlementIdentity>, OrdSet<Arc<RecordedSettlementIdentity>>>(state.downstream.len())?;
            state.downstream.remove(upstream);
        } else {
            admission.ordered_edit::<Arc<RecordedSettlementIdentity>, OrdSet<Arc<RecordedSettlementIdentity>>>(state.downstream.len())?;
            state.downstream.insert(Arc::clone(upstream), targets);
        }
    }
    debit(&mut state.dirty_ordinal_count, row.dirty_ordinals.len())?;
    debit(&mut state.pending_edge_count, row.pending_upstream.len())?;
    debit(
        &mut state.settlement_key_payload_bytes,
        row.posting_payload_bytes,
    )?;
    admission.ordered_remove::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
        state.settlements.len(),
    )?;
    state.settlements.remove(identity);
    Ok(())
}

/// Counters fall only by what an installed row added; underflow is a broken
/// index, reported rather than wrapped.
fn debit<N>(counter: &mut N, amount: N) -> Result<(), CompanionPreflightStop>
where
    N: Copy + std::ops::Sub<Output = N> + PartialOrd,
{
    if amount > *counter {
        return Err(CompanionPreflightStop::PreparationMemoryCounterOverflow);
    }
    *counter = *counter - amount;
    Ok(())
}
