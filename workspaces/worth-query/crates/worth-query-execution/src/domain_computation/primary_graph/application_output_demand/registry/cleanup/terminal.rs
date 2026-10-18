//! Paid detachment of one queued terminal row.
use super::*;
impl DemandRegistryState {
    /// Detach one exact queued row. The caller destroys this custody after
    /// releasing the registry guard, then may advance to the next head.
    pub(super) fn take_terminal_cleanup_step(
        &mut self,
        admission: &mut InvalidationEditAdmission,
        entry_paid: bool,
    ) -> Result<Option<RetiredTerminalCleanup>, WorthQueryOutputDemandDenial> {
        if !entry_paid {
            admission
                .charge_external_work(2)
                .map_err(|_| empty_work_denial())?;
        }
        let Some(head) = self.pending_cleanup_head.as_ref() else {
            return Ok(None);
        };
        // Every reached denial subject in this owner and its record,
        // prerequisite, and settlement index helpers is at most 64 bytes.
        // Fund that one terminal String before any later refusal can build it.
        let terminal_backing = 64usize
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(empty_work_denial)?;
        admission
            .charge_external_work(u64::try_from(terminal_backing).map_err(|_| empty_work_denial())?)
            .map_err(|_| empty_work_denial())?;
        admission
            .admit_read_scratch(u64::try_from(terminal_backing).map_err(|_| empty_work_denial())?)
            .map_err(subject_scratch_denial)?;
        admission
            .charge_external_work(key_work(head)?)
            .map_err(|_| work_denial())?;
        // Pay every reached record-tree descent before unlinking the head.
        // The final removal's node shifts have a separate owner quote.
        for _ in 0..5 {
            self.charge_record_lookup(head.as_ref(), admission)?;
        }
        self.charge_record_removal_movement(admission)?;
        let awaited = super::super::refreshed_rejoin::awaited_by_stale_owner(
            &self.records,
            head.as_ref(),
            admission,
        )?;
        let record = self
            .records
            .get(head.as_ref())
            .expect("queued row stays retained");
        let release_obligations = matches!(record.idle_disposition(awaited), IdleRowDisposition::Retired(reason) if reason.releases_finished_work())
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0
            && record.performed_obligations.capacity() != 0;
        if release_obligations {
            let slot = std::mem::size_of::<super::super::PerformedOutputObligation>();
            let destruction = record
                .performed_obligations
                .capacity()
                .checked_mul(slot)
                .and_then(|backing| {
                    record
                        .performed_obligations
                        .len()
                        .checked_mul(slot)
                        .and_then(|initialized| backing.checked_add(initialized))
                })
                .and_then(|work| work.checked_add(8))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(u64::try_from(destruction).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
        }
        if release_obligations || !record.is_required() {
            for _ in 0..3 {
                self.charge_record_lookup(head.as_ref(), admission)?;
            }
            super::super::required_work::charge_required_key_lookup(
                self,
                head.as_ref(),
                admission,
            )?;
            let levels =
                usize::BITS as usize - self.required_keys.len().max(1).leading_zeros() as usize;
            let movement = levels
                .checked_mul(3)
                .and_then(|nodes| {
                    nodes.checked_mul(super::super::required_members::minimum_member_bytes()?)
                })
                .and_then(|work| work.checked_add(4))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(
                    (u64::try_from(movement).map_err(|_| work_denial())?)
                        .checked_add(1)
                        .ok_or_else(work_denial)?,
                )
                .map_err(|_| work_denial())?;
        }
        let edges = record.prerequisites.len();
        self.admit_record_prerequisite_release(head.as_ref(), admission)?;
        self.settlement_keys.admit_selected_removal_work(
            record.settlements.iter().map(|(identity, _)| identity),
            admission,
        )?;
        let member_count = edges.checked_mul(2).ok_or_else(work_denial)?;
        let member_backing = member_count
            .checked_mul(std::mem::size_of::<
                super::super::required_members::DetachedRequiredMember,
            >())
            .ok_or_else(work_denial)?;
        admission
            .admit_read_scratch(u64::try_from(member_backing).map_err(|_| work_denial())?)
            .map_err(member_scratch_denial)?;
        let mut retired_members = Vec::new();
        if member_count != 0 {
            retired_members
                .try_reserve_exact(member_count)
                .map_err(|_| empty_capacity_denial())?;
            if retired_members.capacity() != member_count {
                return Err(empty_capacity_denial());
            }
        }
        // The queued key must be destroyed before its membership's final
        // capacity ticket is released, even if this removes the row.
        let detached_work = std::mem::size_of::<RetiredTerminalCleanup>()
            .checked_add(std::mem::size_of::<DemandRecord>())
            .and_then(|work| {
                work.checked_add(std::mem::size_of::<Vec<Arc<WorthQueryOutputDemandKey>>>())
            })
            .and_then(|work| work.checked_add(member_backing))
            .and_then(|work| work.checked_add(8))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(detached_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        // This drain immediately inspects the next head, including the empty
        // case. Pay that final header read before unlinking this head so a
        // one-unit-short request leaves the current queued work retriable.
        admission
            .charge_external_work(2)
            .map_err(|_| empty_work_denial())?;
        let member = record.work_membership.as_ref().map(Arc::clone);

        let orphaned_successor = record.successor_is_orphaned(awaited);
        let walks: Vec<_> = record
            .prerequisites
            .iter()
            .map(|upstream| (upstream.as_ref().clone(), 3))
            .chain(std::iter::once((head.as_ref().clone(), 3)))
            .collect();
        let priced = price_mutation(
            self,
            admission,
            &walks,
            super::super::refreshed_rejoin::WalkKind::TerminalCleanup,
        )?;
        Ok(priced.complete(|priced| {
            let state = priced.state_mut();
            let head = state.pending_cleanup_head.take().unwrap();
            let mut record = state.records.get_mut(head.as_ref()).unwrap();
            let next = record.pending_cleanup_next.take();
            record.pending_cleanup_queued = false;
            let retained_key_bytes = std::mem::take(&mut record.pending_cleanup_key_bytes);
            let release = matches!(record.idle_disposition(awaited), IdleRowDisposition::Retired(reason) if reason.releases_finished_work())
                && record.framework_required_count == 0
                && record.prepared_prerequisite_claims == 0;
            let successor = orphaned_successor.then(|| record.held_successor.take()).flatten();
            record.finish();
            state.pending_cleanup_head = next;
            let (released, settlements, checkpoint, release_refund) = if release {
                priced.release_prerequisites_detached(head.as_ref(), &mut retired_members)
            } else {
                (Vec::new(), Vec::new(), None, 0)
            };
            let state = priced.state_mut();
            let (obligations, refund_obligation_bytes) = if release_obligations {
                let mut record = state.records.get_mut(head.as_ref()).unwrap();
                let refund = record.obligation_reserved_bytes();
                let obligations = std::mem::take(&mut record.performed_obligations);
                (obligations, refund)
            } else {
                (Vec::new(), 0)
            };
            let own_member = priced.release_member(head.as_ref());
            let state = priced.state_mut();
            let refund_required_bytes = retained_key_bytes
                .checked_add(release_refund)
                .and_then(|bytes| bytes.checked_add(own_member.refund_required_bytes))
                .expect("admitted terminal cleanup charge fits");
            let removed = if state.records.get(head.as_ref()).is_some_and(|record| {
                matches!(record.idle_disposition(awaited), IdleRowDisposition::Retired(reason) if reason.releases_finished_work())
                    && record.interests == 0
                    && !record.is_required()
                    && record.performed_source.is_none()
                    && !record.pending_cleanup_queued
            }) {
                match state.records.get(head.as_ref()).and_then(|row| row.closed_cleanup.queued_note()) { Some(note) => {
                    // This paid terminal step completes its own closed note;
                    // unrelated stale notes remain the optional drain's debt.
                    state.take_closed_note(note).prune(|state| state.remove_record(head.as_ref()))
                } _ => { state.remove_record(head.as_ref()) }}
            } else {
                None
            };
            let newest_member = priced.release_newest_member(head.as_ref());
            let refund_required_bytes = refund_required_bytes.checked_add(newest_member.refund_required_bytes).expect("admitted newest member refund fits");
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
                refund_required_bytes,
                refund_obligation_bytes,
            }))
        }))
    }
}

fn subject_scratch_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
        | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    empty_work_denial()
                }
                _ => empty_capacity_denial(),
            }
}

fn member_scratch_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
        | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => empty_capacity_denial(),
            }
}

#[cfg(test)]
mod tests;
