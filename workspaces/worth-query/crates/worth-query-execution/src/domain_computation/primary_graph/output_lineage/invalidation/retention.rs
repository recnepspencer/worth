//! Retained capacity follows the allocation that owns the bytes. The versions
//! of a branch are persistent: each shares with its predecessor every node
//! its own edits did not copy. A version reserves what its edits copied, and
//! the root reserves what its oldest version shares with versions that left.

use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::execution_runtime::source_invalidation::{
    RetainedInvalidationCapacity, WorthQueryInvalidationResourceDenial,
    WorthQueryInvalidationResources,
};

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::edit_admission::RetainedIndexCheckpoint;
use super::fact_key::FactPostingKey;
use super::index_capacity::{arc_bytes, retained_forest_bytes, retained_map_bytes};
use super::mark_state::{EqualOutputLink, FactPosting, MarkState, SettlementMarks};
use super::source_alignment::{BranchMarkRoot, HistoricalMarkState};
use super::InvalidationEditAdmission;

type Capacity = Option<Arc<RetainedInvalidationCapacity>>;

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

/// One version's whole index, as if it shared no node. No inventory
/// traversal: logical cardinalities are changed with their selected
/// posting/mark edits.
pub(super) fn state_bound(state: &MarkState) -> Option<u64> {
    let rows = state.settlements.len();
    let postings = state.posting_count;
    arc_bytes::<MarkState>()?
        .checked_add(state.key_payload_bytes)?
        .checked_add(state.settlement_key_payload_bytes)?
        // A whole-index quote includes one live basis allocation per row.
        // Its actual ticket remains separately owned by that allocation.
        .checked_add(
            state
                .maximum_basis_allocation_bytes
                .checked_mul(rows as u64)?,
        )?
        // Each row owns a consumed_upstream set, even when empty.
        .checked_add(
            retained_forest_bytes::<Arc<RecordedSettlementIdentity>, ()>(
                state.downstream_edge_count,
                rows,
            )?,
        )?
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
            // One posting_ordinals map per row; each distinct key has at
            // least one ordinal, so posting_count bounds their total keys.
            retained_forest_bytes::<Arc<FactPostingKey>, im::OrdSet<usize>>(postings, rows)?,
        )?
        // Each row/key pair owns its own ordinal set. Their root count is
        // at most posting_count; combining them into one tree undercounts.
        .checked_add(retained_forest_bytes::<usize, ()>(postings, postings)?)?
        .checked_add(retained_map_bytes::<
            Arc<FactPostingKey>,
            im::OrdSet<FactPosting>,
        >(state.postings.len())?)?
        .checked_add(retained_forest_bytes::<FactPosting, ()>(
            postings,
            state.postings.len(),
        )?)?
        .checked_add(retained_map_bytes::<
            Arc<RecordedSettlementIdentity>,
            Arc<SettlementMarks>,
        >(state.settlements.len())?)?
        .checked_add(retained_map_bytes::<
            Arc<RecordedSettlementIdentity>,
            im::OrdSet<Arc<RecordedSettlementIdentity>>,
        >(state.downstream.len())?)?
        .checked_add(
            retained_forest_bytes::<Arc<RecordedSettlementIdentity>, ()>(
                state.downstream_edge_count,
                state.downstream.len(),
            )?,
        )?
        .checked_add(retained_forest_bytes::<usize, ()>(
            state.dirty_ordinal_count,
            rows,
        )?)?
        .checked_add(
            retained_forest_bytes::<Arc<RecordedSettlementIdentity>, ()>(
                state.pending_edge_count,
                rows,
            )?,
        )?
        .checked_add(arc_bytes::<SettlementMarks>()?.checked_mul(state.settlements.len() as u64)?)
}

/// What a reservation holds beside its own ticket.
fn reserved(capacity: &Capacity) -> Option<u64> {
    let ticket = arc_bytes::<RetainedInvalidationCapacity>()?;
    Some(
        capacity
            .as_ref()
            .map_or(0, |capacity| capacity.bytes().saturating_sub(ticket)),
    )
}

/// Admits a branch's first version, which has no predecessor to share with.
pub(super) fn admit_first(
    state: &mut MarkState,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    admit(state, u64::MAX, resources, admission)
}

/// Admits a version beside its predecessor, which stays retained under its
/// own reservation. `copied` is what the version's edits were admitted for:
/// the nodes it does not share.
pub(super) fn admit_version(
    state: &mut MarkState,
    copied: u64,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let overflow = CompanionPreflightStop::PreparationMemoryCounterOverflow;
    let own = arc_bytes::<MarkState>()
        .and_then(|version| version.checked_add(copied))
        .ok_or(overflow)?;
    admit(state, own, resources, admission)
}

/// Admits a version that replaces the live one at its source position, so
/// what the replaced version copied lives on in it. `before` is the
/// admission's retained-index subtotal when the edit began. Preparation scratch
/// and allocations that have separate tickets are not copied index storage.
pub(super) fn admit_replacement(
    state: &mut MarkState,
    before: RetainedIndexCheckpoint,
    resources: &WorthQueryInvalidationResources,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let overflow = CompanionPreflightStop::PreparationMemoryCounterOverflow;
    let copied = admission.index_bytes_since(before)?;
    // The clone still carries the reservation of the version it replaces.
    let own = reserved(&state.retained_capacity)
        .and_then(|replaced| replaced.checked_add(copied))
        .ok_or(overflow)?;
    admit(state, own, resources, admission)
}

/// A version never needs more than its whole index.
fn admit(
    state: &mut MarkState,
    own: u64,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let whole =
        state_bound(state).ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    state.retained_capacity = Some(reserve(resources, own.min(whole), admission)?);
    Ok(())
}

/// Reserve a newly edited history map beside every pinned predecessor. Edited
/// maps keep the conservative whole-map bound; unchanged replacements retain
/// their existing allocation owner's ticket instead.
pub(super) fn admit_edited_history(
    root: &mut BranchMarkRoot,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    use worth_relational::facade::publication::PatchStreamPosition;
    let bytes =
        retained_map_bytes::<Option<PatchStreamPosition>, HistoricalMarkState>(root.past.len())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    root.history_capacity = Some(reserve(resources, bytes, admission)?);
    Ok(())
}

/// A root owns its object, and what its oldest version shares
/// with versions that have left. `left` is the version leaving with this
/// root: its successor still shares what it had reserved, and with its own
/// reservation never needs more than its whole index. Its history ticket is
/// shared with unchanged replacements; the delivery owner separately admits
/// every insertion/removal before calling this function.
pub(super) fn admit_root(
    root: &mut BranchMarkRoot,
    left: Option<&MarkState>,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let bytes = arc_bytes::<BranchMarkRoot>()
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    root.retained_capacity = Some(reserve(resources, bytes, admission)?);
    if root.history_capacity.is_none() {
        admit_edited_history(root, resources, admission)?;
    }
    let overflow = CompanionPreflightStop::PreparationMemoryCounterOverflow;
    let oldest = root
        .past
        .get_min()
        .map_or(&root.current, |(_, past)| &past.state);
    let unreserved = state_bound(oldest)
        .zip(reserved(&oldest.retained_capacity))
        .map(|(whole, own)| whole.saturating_sub(own))
        .ok_or(overflow)?;
    let inherited = reserved(&root.inherited_capacity).ok_or(overflow)?;
    let shared = left
        .map_or(Some(0), |left| reserved(&left.retained_capacity))
        .and_then(|left| left.checked_add(inherited))
        .ok_or(overflow)?
        .min(unreserved);
    if shared != inherited {
        root.inherited_capacity = match shared {
            0 => None,
            shared => Some(reserve(resources, shared, admission)?),
        };
    }
    Ok(())
}

#[cfg(test)]
mod budget_tests;
