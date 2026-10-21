//! Continuation storage funded before every member of a wave is prepared.
use super::*;

pub(in super::super) struct PreparedContinuationWave<'owner, Schema: ApplicationSchema> {
    custody: &'owner mut RequiredContinuations<Schema>,
    // Backing dies before its ledger ticket, including on an unused reservation.
    replacement: Option<Vec<RequiredFreshProgress<Schema>>>,

    reserved: usize,
}

impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(in super::super) fn prepare_wave(&mut self) -> PreparedContinuationWave<'_, Schema> {
        PreparedContinuationWave {
            custody: self,
            replacement: None,

            reserved: 0,
        }
    }
}

impl<Schema: ApplicationSchema + 'static> PreparedContinuationWave<'_, Schema> {
    /// Scheduling only: the actual successor must still resume and certify.
    pub(in super::super) fn retains_refresh(
        &self,
        selected: &SelectedReadyReadmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.custody
            .refresh_position(selected)
            .map(|index| index.is_some())
    }

    pub(in super::super) fn awaited_delivery(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<Option<awaited_delivery::AwaitedDelivery<'_, Schema>>, WorthQueryOutputDemandDenial>
    {
        self.custody.awaited_delivery(registry)
    }

    pub(in super::super) fn entries(&self) -> &[RequiredFreshProgress<Schema>] {
        &self.custody.entries
    }

    pub(in super::super) fn last_mut(&mut self) -> &mut RequiredFreshProgress<Schema> {
        self.custody
            .entries
            .last_mut()
            .expect("a member was installed")
    }

    /// Discard a completed stale plan before inspecting its replaced Ready.
    /// The successor's final interest restores that Ready through the registry.
    pub(in super::super) fn discard_last(&mut self) {
        drop(
            self.custody
                .entries
                .pop()
                .expect("a refused member was installed"),
        );
    }

    /// Fund both the future comparison/write and its physical backing before
    /// preparation can replace an interest. Applying a completed member cannot
    /// need an allocation, a capacity admission or another work admission.
    pub(in super::super) fn reserve_one(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if self.reserved == 0 {
            let slot = self.custody.prepare_slot(registry)?;
            self.replacement = slot.replacement;
        }
        let next_len = self
            .custody
            .entries
            .len()
            .checked_add(self.reserved)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(capacity_denial)?;
        let available = self
            .replacement
            .as_ref()
            .map_or(self.custody.entries.capacity(), Vec::capacity);
        if next_len > available {
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next_len)
                .map_err(|_| capacity_denial())?;
            self.replacement = Some(replacement);
        }
        self.reserved += 1;
        Ok(())
    }

    pub(in super::super) fn release_unused(&mut self) {
        assert!(self.reserved > 0, "one provisional slot was reserved");
        self.reserved -= 1;
        if self.reserved == 0 {
            drop(self.replacement.take());
        }
    }

    pub(in super::super) fn install_next(
        &mut self,
        progress: RequiredFreshProgress<Schema>,
    ) -> RequiredFreshOutcome {
        assert!(
            self.reserved > 0,
            "one funded slot for each prepared member"
        );
        self.reserved -= 1;
        // A successor releases the entry it supersedes before deciding whether
        // the provisional backing is still needed. Unused growth retains no
        // capacity after the real owner-to-owner handoff.
        let newest = progress.successor.interest();
        self.custody
            .entries
            .retain(|entry| !newest.supersedes(entry.successor.interest()));
        if self.custody.entries.len() + self.reserved < self.custody.entries.capacity() {
            drop(self.replacement.take());
        }
        if let Some(mut replacement) = self.replacement.take() {
            replacement.append(&mut self.custody.entries);
            let retired = std::mem::replace(&mut self.custody.entries, replacement);
            drop(retired);
        }
        assert!(self.custody.entries.len() < self.custody.entries.capacity());
        self.custody.entries.push(progress);
        self.custody
            .entries
            .last_mut()
            .expect("installed member")
            .take_outcome()
    }
}
