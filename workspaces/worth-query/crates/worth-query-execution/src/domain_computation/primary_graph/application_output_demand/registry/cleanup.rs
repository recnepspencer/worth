pub(super) mod closed;
pub(super) mod closed_notes;
pub(super) use closed_notes::ClosedCleanupLinks;
pub(super) use closed_notes::ClosedCleanupQueue;
pub(super) mod custody;
mod custody_notes;

pub(super) use custody_notes::CustodyCleanupLinks;
pub(super) use custody_notes::CustodyCleanupQueue;
mod drain;

mod terminal;

use std::sync::Arc;

use super::DemandRecord;
use super::DemandRegistryState;
use super::IdleRowDisposition;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

enum CleanupTarget {
    Allocations,
    ReleasedMember,
}

impl DemandRegistryState {
    /// The selected Arc already owns this key. Linking it into the registry is
    /// allocation free, so a post-effect pin release can only defer cleanup.
    pub(super) fn defer_terminal_cleanup(&mut self, key: &Arc<WorthQueryOutputDemandKey>) -> bool {
        // A note is conservative; the paid drain resolves whether a stale owner still awaits it.
        if !self
            .records
            .get(key.as_ref())
            .is_some_and(DemandRecord::terminal)
        {
            return false;
        }
        let awaited = false;
        self.defer_terminal_cleanup_with_awaited(key, awaited)
    }

    pub(super) fn defer_terminal_cleanup_with_awaited(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,

        awaited: bool,
    ) -> bool {
        self.defer_terminal_cleanup_target(key, awaited, CleanupTarget::Allocations)
    }

    /// A released funded member also owes retirement of its empty terminal row.
    pub(super) fn defer_released_member_cleanup(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        awaited: bool,
    ) -> bool {
        self.defer_terminal_cleanup_target(key, awaited, CleanupTarget::ReleasedMember)
    }

    fn defer_terminal_cleanup_target(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,

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
            // A closed-row note already owns this member's retirement. Keep
            // that occurrence debt with its paid cascade, rather than making
            // an own cascade wait on a second terminal-cleanup owner.
            || (matches!(target, CleanupTarget::ReleasedMember) && record.closed_cleanup.queued())
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
    _settlements: Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    >,
    _retired_members: Vec<super::required_members::DetachedRequiredMember>,
    _obligations: Vec<super::PerformedOutputObligation>,
    _own_member: super::required_members::DetachedRequiredMember,
    _newest_member: super::required_members::DetachedRequiredMember,
    _removed: Option<DemandRecord>,
    _head: Arc<WorthQueryOutputDemandKey>,
    _member: Option<Arc<super::required_work::RequiredWorkMembership>>,

    _successor: Option<super::held_successor::HeldRequiredSuccessor>,
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "queued terminal prerequisite cleanup exceeds request work",
    )
}

fn empty_capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}
