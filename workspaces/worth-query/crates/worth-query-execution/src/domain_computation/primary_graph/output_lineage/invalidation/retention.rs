use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::execution_runtime::source_invalidation::{
    RetainedInvalidationCapacity, WorthQueryInvalidationResourceDenial,
    WorthQueryInvalidationResources,
};

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::fact_key::FactPostingKey;
use super::index_capacity::{arc_bytes, retained_map_bytes};
use super::mark_state::{EqualOutputLink, FactPosting, MarkState, SettlementMarks};
use super::source_alignment::{BranchMarkRoot, HistoricalMarkState};

pub(super) fn reserve(
    resources: &WorthQueryInvalidationResources,
    bytes: u64,
    admission: &mut impl IndexAdmission,
) -> Result<Arc<RetainedInvalidationCapacity>, CompanionPreflightStop> {
    let ticket_bytes = arc_bytes::<RetainedInvalidationCapacity>()
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.bytes(ticket_bytes)?;
    let bytes = bytes
        .checked_add(ticket_bytes)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    let capacity = resources
        .reserve_retained_capacity(bytes)
        .map_err(|denial| match denial {
            WorthQueryInvalidationResourceDenial::RetentionCapacityExhausted {
                requested,
                retained,
                maximum,
            } => CompanionPreflightStop::RetainedCompanionCapacityExhausted {
                requested,
                retained,
                maximum,
            },
            _ => unreachable!("installed resources have positive capacities"),
        })?;
    Ok(Arc::new(capacity))
}

/// No inventory traversal: logical cardinalities are changed with their
/// selected posting/mark edits. Every live version reserves its own conservative
/// node-capacity bound; physical sharing is deliberately not claimed as freed.
pub(super) fn state_bound(state: &MarkState) -> Option<u64> {
    arc_bytes::<MarkState>()?
        .checked_add(state.key_payload_bytes)?
        .checked_add(state.settlement_key_payload_bytes)?
        .checked_add(retained_map_bytes::<Arc<RecordedSettlementIdentity>, ()>(
            state
                .downstream_edge_count
                .checked_add(state.settlements.len())?,
        )?)?
        .checked_add(retained_map_bytes::<
            Arc<RecordedSettlementIdentity>,
            Arc<EqualOutputLink>,
        >(state.equal_links.len())?)?
        .checked_add(arc_bytes::<EqualOutputLink>()?.checked_mul(state.equal_links.len() as u64)?)?
        .checked_add(
            arc_bytes::<RecordedSettlementIdentity>()?
                .checked_mul(state.equal_links.len().checked_mul(2)? as u64)?,
        )?
        .checked_add(
            arc_bytes::<RecordedSettlementIdentity>()?.checked_mul(
                state
                    .downstream_edge_count
                    .checked_add(state.settlements.len())? as u64,
            )?,
        )?
        .checked_add(
            retained_map_bytes::<Arc<FactPostingKey>, im::OrdSet<usize>>(
                state.posting_count.checked_add(state.settlements.len())?,
            )?,
        )?
        .checked_add(retained_map_bytes::<usize, ()>(
            state.posting_count.checked_add(state.settlements.len())?,
        )?)?
        .checked_add(retained_map_bytes::<
            Arc<FactPostingKey>,
            im::OrdSet<FactPosting>,
        >(state.postings.len())?)?
        .checked_add(retained_map_bytes::<FactPosting, ()>(
            state.posting_count.checked_add(state.postings.len())?,
        )?)?
        .checked_add(retained_map_bytes::<
            Arc<RecordedSettlementIdentity>,
            Arc<SettlementMarks>,
        >(state.settlements.len())?)?
        .checked_add(retained_map_bytes::<
            Arc<RecordedSettlementIdentity>,
            im::OrdSet<Arc<RecordedSettlementIdentity>>,
        >(state.downstream.len())?)?
        .checked_add(retained_map_bytes::<Arc<RecordedSettlementIdentity>, ()>(
            state
                .downstream_edge_count
                .checked_add(state.downstream.len())?,
        )?)?
        .checked_add(retained_map_bytes::<usize, ()>(
            state
                .dirty_ordinal_count
                .checked_add(state.settlements.len())?,
        )?)?
        .checked_add(retained_map_bytes::<Arc<RecordedSettlementIdentity>, ()>(
            state
                .pending_edge_count
                .checked_add(state.settlements.len())?,
        )?)?
        .checked_add(arc_bytes::<SettlementMarks>()?.checked_mul(state.settlements.len() as u64)?)
}

pub(super) fn admit_state(
    state: &mut MarkState,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    state.retained_capacity = Some(reserve(
        resources,
        state_bound(state).ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        admission,
    )?);
    Ok(())
}

pub(super) fn admit_root(
    root: &mut BranchMarkRoot,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    use worth_relational::facade::publication::PatchStreamPosition;
    let bytes = arc_bytes::<BranchMarkRoot>()
        .and_then(|n| {
            n.checked_add(retained_map_bytes::<
                Option<PatchStreamPosition>,
                HistoricalMarkState,
            >(root.past.len())?)
        })
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    root.retained_capacity = Some(reserve(resources, bytes, admission)?);
    Ok(())
}
