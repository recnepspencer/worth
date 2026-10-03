//! The checkpoint fold moves its independently funded roster into the ledger.
use super::*;
use std::sync::Arc;

impl SelectedReleaseHeadRoster {
    #[cfg(test)]
    pub(in super::super) fn prefix_fixture(&self, additional: usize) -> Self {
        let mut entries = self.entries.clone();
        entries.try_reserve_exact(additional).unwrap();
        Self {
            root: self.root,
            entries,
            allocation_custody: None,
        }
    }
    pub(in super::super) fn prepare_fold_capacity(
        &mut self,
        count: usize,
        window: &mut super::super::backing::LiveBackingWindow<'_>,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let additional = count.saturating_sub(self.entries.len());
        window.grow_vec(&mut self.entries, additional)
    }

    pub(in super::super) fn retain_fold_charge(
        &mut self,
        custody: Arc<super::super::backing::LiveReleaseAllocation>,
    ) {
        self.allocation_custody = Some(custody);
    }

    pub(in super::super) fn copy_into_preallocated(
        &self,
        target: &mut Self,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        if target.entries.capacity() < self.entries.len() {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        target.entries.clear();
        target.entries.extend_from_slice(&self.entries);
        target.root = self.root;
        Ok(())
    }
}

impl SelectedReleaseHeadStep {
    pub(in super::super) fn apply_preallocated(
        self,
        roster: &mut SelectedReleaseHeadRoster,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let transition = self.prepare(roster)?;
        if !transition.has_backing(roster) {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        transition.apply(roster);
        Ok(())
    }
}
