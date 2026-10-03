//! The checkpoint fold moves its roster, inside the capture envelope, into the ledger.
use super::*;

impl SelectedReleaseHeadRoster {
    #[cfg(test)]
    pub(in super::super) fn prefix_fixture(&self, additional: usize) -> Self {
        let mut entries = self.entries.clone();
        entries.try_reserve_exact(additional).unwrap();
        Self {
            root: self.root,
            entries,
        }
    }
    /// An empty fold target at the envelope's admitted head population.
    pub(in super::super) fn with_fold_capacity(count: usize) -> Self {
        Self {
            root: None,
            entries: Vec::with_capacity(count),
        }
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
