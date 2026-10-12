pub(super) mod closed;
pub(super) mod closed_notes;
pub(super) use closed_notes::{ClosedCleanupLinks, ClosedCleanupQueue};
pub(super) mod custody;
mod custody_notes;
pub(super) mod custody_price;
pub(super) use custody_notes::{CustodyCleanupLinks, CustodyCleanupQueue};
mod drain;
mod key_comparison;
mod prerequisite_release;
mod terminal;
use super::refreshed_rejoin::price_mutation;
use key_comparison::key_work;

use std::sync::Arc;

use super::{
    DemandRecord, DemandRegistryState, IdleRowDisposition, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

enum CleanupTarget {
    Allocations,
    ReleasedMember,
}

impl DemandRegistryState {
    /// The selected Arc already owns this key. Linking it into the registry is
    /// allocation free, so a post-effect pin release can only defer cleanup.
    pub(super) fn defer_terminal_cleanup(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        retained_key_bytes: usize,
    ) -> bool {
        // A note is conservative; the paid drain resolves whether a stale owner still awaits it.
        if !self
            .records
            .get(key.as_ref())
            .is_some_and(DemandRecord::terminal)
        {
            return false;
        }
        let awaited = false;
        self.defer_terminal_cleanup_with_awaited(key, retained_key_bytes, awaited)
    }

    pub(super) fn defer_terminal_cleanup_with_awaited(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        retained_key_bytes: usize,
        awaited: bool,
    ) -> bool {
        self.defer_terminal_cleanup_target(
            key,
            retained_key_bytes,
            awaited,
            CleanupTarget::Allocations,
        )
    }

    /// A released funded member also owes retirement of its empty terminal row.
    pub(super) fn defer_released_member_cleanup(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        awaited: bool,
    ) -> bool {
        self.defer_terminal_cleanup_target(key, 0, awaited, CleanupTarget::ReleasedMember)
    }

    fn defer_terminal_cleanup_target(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        retained_key_bytes: usize,
        awaited: bool,
        target: CleanupTarget,
    ) -> bool {
        let Some(mut record) = self.records.get_mut(key.as_ref()) else {
            return false;
        };
        match record.idle_disposition(awaited) {
            IdleRowDisposition::InFlight
            | IdleRowDisposition::KeptAnswer
            | IdleRowDisposition::HeldForStaleOwner
                if !record.successor_is_orphaned(awaited) =>
            {
                return false
            }
            IdleRowDisposition::InFlight
            | IdleRowDisposition::KeptAnswer
            | IdleRowDisposition::HeldForStaleOwner => {}
            IdleRowDisposition::Retired(reason) if reason.releases_finished_work() => {}
            IdleRowDisposition::Retired(_) => return false,
        }
        if record.framework_required_count != 0
            || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued
            || (matches!(target, CleanupTarget::Allocations)
                && record.prerequisites.capacity() == 0
                && record.checkpoint_prerequisites.is_none()
                && record.settlements.capacity() == 0
                && record.performed_obligations.capacity() == 0
                && !matches!(
                    record.idle_disposition(awaited),
                    IdleRowDisposition::Retired(super::RowRetirement::AbandonedPublication)
                )
                && !record.successor_is_orphaned(awaited))
        {
            return false;
        }
        let closed_note = record.closed_cleanup.queued_note();
        record.pending_cleanup_next = self.pending_cleanup_head.take();
        record.pending_cleanup_queued = true;
        record.pending_cleanup_key_bytes = retained_key_bytes;
        record.finish();
        self.pending_cleanup_head = Some(Arc::clone(key));
        if let Some(note) = closed_note {
            self.take_closed_note(note).prune(|_| ());
        }
        true
    }
}

/// Field order preserves the key and removed row through their physical
/// destruction before the retained membership can refund its capacity.
pub(super) struct RetiredTerminalCleanup {
    _released: Vec<Arc<WorthQueryOutputDemandKey>>,
    _checkpoint: Option<super::prerequisite_claims::CheckpointPrerequisiteClaims>,
    _settlements: Vec<(
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
        usize,
    )>,
    _retired_members: Vec<super::required_members::DetachedRequiredMember>,
    _obligations: Vec<super::PerformedOutputObligation>,
    _own_member: super::required_members::DetachedRequiredMember,
    _newest_member: super::required_members::DetachedRequiredMember,
    _removed: Option<DemandRecord>,
    _head: Arc<WorthQueryOutputDemandKey>,
    _member: Option<Arc<super::required_work::RequiredWorkMembership>>,
    pub(super) refund_required_bytes: usize,
    pub(super) refund_obligation_bytes: usize,
    _successor: Option<super::held_successor::HeldRequiredSuccessor>,
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "queued terminal prerequisite cleanup exceeds request work",
    )
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}
