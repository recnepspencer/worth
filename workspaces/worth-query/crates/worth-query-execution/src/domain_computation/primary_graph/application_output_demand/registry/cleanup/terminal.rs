//! Detach one queued terminal row; destroy its custody outside the guard.
use super::*;
impl DemandRegistryState {
    pub(super) fn take_terminal_cleanup_step(
        &mut self,
    ) -> Result<Option<RetiredTerminalCleanup>, WorthQueryOutputDemandDenial> {
        let Some(head) = self.pending_cleanup_head.as_ref() else {
            return Ok(None);
        };
        let awaited =
            super::super::refreshed_rejoin::awaited_by_stale_owner(&self.records, head.as_ref());
        let record = self
            .records
            .get(head.as_ref())
            .expect("queued row stays retained");
        let release = matches!(record.idle_disposition(awaited), IdleRowDisposition::Retired(reason) if reason.releases_finished_work())
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0;
        let release_obligations = release && record.performed_obligations.capacity() != 0;
        let member_count = record
            .prerequisites
            .len()
            .checked_mul(2)
            .ok_or_else(work_denial)?;
        let mut retired_members = Vec::new();
        retired_members
            .try_reserve_exact(member_count)
            .map_err(|_| empty_capacity_denial())?;
        let member = record.work_membership.as_ref().map(Arc::clone);
        let orphaned_successor = record.successor_is_orphaned(awaited);
        let head = self
            .pending_cleanup_head
            .take()
            .expect("queued head exists");
        let successor = {
            let mut record = self
                .records
                .get_mut(head.as_ref())
                .expect("queued row stays retained");
            self.pending_cleanup_head = record.pending_cleanup_next.take();
            record.pending_cleanup_queued = false;
            let successor = orphaned_successor
                .then(|| record.held_successor.take())
                .flatten();
            record.finish();
            successor
        };
        let (released, settlements, checkpoint) = if release {
            self.release_record_prerequisites_detached(head.as_ref(), &mut retired_members)
        } else {
            (Vec::new(), Vec::new(), None)
        };
        let obligations = if release_obligations {
            std::mem::take(
                &mut self
                    .records
                    .get_mut(head.as_ref())
                    .unwrap()
                    .performed_obligations,
            )
        } else {
            Vec::new()
        };
        let own_member = self.remove_required_member_if_released_detached(head.as_ref());
        let removable = self.records.get(head.as_ref()).is_some_and(|record| {
            matches!(record.idle_disposition(awaited), IdleRowDisposition::Retired(reason) if reason.releases_finished_work())
                && record.interests == 0 && !record.is_required() && record.performed_source.is_none()
                && !record.pending_cleanup_queued
        });
        let removed = if removable {
            match self
                .records
                .get(head.as_ref())
                .and_then(|row| row.closed_cleanup.queued_note())
            {
                Some(note) => self
                    .take_closed_note(note)
                    .prune(|state| state.remove_record(head.as_ref())),
                None => self.remove_record(head.as_ref()),
            }
        } else {
            None
        };
        let newest_member = self.release_newest_member(head.as_ref());
        Ok(Some(RetiredTerminalCleanup {
            _newest_member: newest_member,
            _successor: successor,
            _released: released,
            _settlements: settlements,
            _checkpoint: checkpoint,
            _retired_members: retired_members,
            _obligations: obligations,
            _own_member: own_member,
            _removed: removed,
            _head: head,
            _member: member,
        }))
    }
}
