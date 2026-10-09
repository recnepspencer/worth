//! Borrow the exact successor and predecessor while caller custody is live.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

impl<Schema> RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn interest(
        &self,
    ) -> &WorthQueryOutputDemandInterest {
        self.successor.interest()
    }

    pub(in crate::domain_computation::primary_graph) fn producer_identity(&self) -> &str {
        self.successor.producer_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn producer_contacts(&self) -> usize {
        self.successor.producer_contacts()
    }

    pub(in crate::domain_computation::primary_graph) fn advance_checkpoint(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.successor
            .advance_checkpoint(phase, runtime, request, admission)
    }

    pub(in crate::domain_computation::primary_graph) fn is_family<Family>(&self) -> bool
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    {
        self.successor.family_type() == TypeId::of::<Family>()
    }

    pub(in crate::domain_computation::primary_graph) fn predecessor(
        &self,
    ) -> &SelectedReadyReadmission {
        self.successor.predecessor()
    }

    pub(super) fn take_outcome(&mut self) -> RequiredFreshOutcome {
        self.outcome
            .take()
            .expect("required successor outcome is taken only after custody install")
    }
}

impl<'caller, Schema> PreparedRequiredContinuationSlot<'caller, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in super::super) fn installation_work(&self) -> u64 {
        self.installation_work
    }

    pub(in super::super) fn entries(&self) -> &[RequiredFreshProgress<Schema>] {
        &self.caller.entries
    }

    /// Install `progress` as the last entry, ending any it supersedes.
    pub(super) fn push(
        mut self,
        progress: RequiredFreshProgress<Schema>,
    ) -> &'caller mut RequiredContinuations<Schema> {
        // A newer successor of the same occurrence, from a newer source or a
        // second refresh of the same row, ends the custody of the older one:
        // dropping its typed demand releases what only it held.
        let newest = progress.successor.interest();
        self.caller
            .entries
            .retain(|entry| !newest.supersedes(entry.successor.interest()));
        // The slot it freed holds the newer one; growth prepared for an
        // additional entry ends unused with its ticket.
        if self.caller.entries.len() < self.caller.entries.capacity() {
            drop(self.replacement.take());
            drop(self.capacity.take());
        }
        if let Some(mut replacement) = self.replacement.take() {
            replacement.append(&mut self.caller.entries);
            let retired_entries = std::mem::replace(&mut self.caller.entries, replacement);
            let retired_capacity =
                std::mem::replace(&mut self.caller.capacity, self.capacity.take());
            // The retired Vec backing dies before its final credit is refunded.
            drop(retired_entries);
            drop(retired_capacity);
        }
        let caller = self.caller;
        caller.entries.push(progress);
        caller
    }
}

impl<Schema> PreparedRequiredContinuationSlot<'_, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Install `progress` and take its outcome.
    pub(in super::super) fn install(
        self,
        progress: RequiredFreshProgress<Schema>,
    ) -> RequiredFreshOutcome {
        self.push(progress)
            .entries
            .last_mut()
            .expect("prepared slot installs one successor")
            .take_outcome()
    }
}

impl<Schema> RequiredContinuations<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// End each entry whose row went back to the Ready it reopened. However
    /// its refresh stopped, refused at admission, interrupted while it
    /// advanced, or stopped when resumed, the next wave claims that refresh
    /// again; ending the entry first frees its custody for that claim. An
    /// entry whose row another advance superseded follows that refresh
    /// first, as its dependent's own demand would.
    pub(super) fn end_restored(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut index = 0;
        while let Some(entry) = self.entries.get_mut(index) {
            entry.successor.follow_refresh(registry, admission)?;
            let restored = registry
                .interest_ready_readmission(entry.interest(), admission)?
                .is_some_and(|ready| {
                    ready
                        .completion()
                        .same_cell(entry.predecessor().completion())
                });
            if restored {
                drop(self.entries.remove(index));
            } else {
                index += 1;
            }
        }
        Ok(())
    }
    /// End every entry. Whether one of them had published: its row answers
    /// Ready on its own, and the predecessor Ready it kept for the wave is
    /// custody the next advance does not hold. The stop of a lookup the
    /// request cannot pay for keeps the entries.
    pub(in super::super) fn end_all(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut published = false;
        for entry in &self.entries {
            let ready = registry.interest_ready_readmission(entry.interest(), admission)?;
            published |= ready.is_some_and(|ready| {
                !ready
                    .completion()
                    .same_cell(entry.predecessor().completion())
            });
        }
        drop(self.take_all());
        Ok(published)
    }

    /// A queue frame refused required custody does not wait holding it, as
    /// a caller does not: the refreshes this wave's frames carried end here.
    /// Each unpublished row goes back to the Ready it replaced, so every
    /// wave that meets the same refusal leaves the same rows behind it. No
    /// lookup precedes the end, so no request work can keep the entries.
    pub(in super::super) fn end_refused(&mut self, stop: &WorthQueryOutputDemandDenial) {
        if refused_custody(stop) {
            drop(self.take_all());
        }
    }
}

/// A retryable refusal of required custody.
pub(super) fn refused_custody(stop: &WorthQueryOutputDemandDenial) -> bool {
    stop.kind() == super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        && stop.recovery_posture()
            == crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
}
