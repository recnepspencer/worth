//! Borrow the exact successor and predecessor while caller custody is live.

use super::*;

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
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.successor
            .advance_checkpoint(runtime, request, admission)
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
    /// again; ending the entry first frees its custody for that claim.
    pub(super) fn end_restored(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut index = 0;
        while let Some(entry) = self.entries.get(index) {
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
    /// custody the next advance does not hold. `None`, with the entries
    /// kept, when the request's work cannot pay for the lookups.
    pub(in super::super) fn end_all(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<bool> {
        let mut published = false;
        for entry in &self.entries {
            let ready = registry
                .interest_ready_readmission(entry.interest(), admission)
                .ok()?;
            published |= ready.is_some_and(|ready| {
                !ready
                    .completion()
                    .same_cell(entry.predecessor().completion())
            });
        }
        drop(self.take_all());
        Some(published)
    }
}
