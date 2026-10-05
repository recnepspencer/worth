use super::page_observation::PageObservationFailure;

/// The manifest entries recovery may still observe. A charge counts what the
/// workload wrote: a root, the entries a full observation finds in leaves,
/// the entries a root step's member declares, one for a point lookup. Where
/// records landed in a tree, and how many blocks a read crosses, charge
/// nothing, so one workload charges the same entries on every run.
pub(super) struct ManifestEntryBudget {
    admitted: u64,
    observed: u64,
    /// The count the latest refused charge would have reached.
    refused_at: Option<u64>,
}

/// A root was charged its one entry. Whoever enumerates a root's leaves is
/// handed this, so no full observation can leave the root's own unit out.
pub(super) struct RootUnit(());

impl ManifestEntryBudget {
    pub(super) const fn new(admitted: u64, already_observed: u64) -> Self {
        Self {
            admitted,
            observed: already_observed,
            refused_at: None,
        }
    }

    pub(super) fn consume(&mut self, entries: usize) -> Result<(), PageObservationFailure> {
        self.consume_with_evidence(entries)
            .map_err(|(observed, _)| self.refuse(observed))
    }

    /// Charges the one entry a root costs, before its leaves are read.
    pub(super) fn charge_root(&mut self) -> Result<RootUnit, PageObservationFailure> {
        self.consume(1).map(|()| RootUnit(()))
    }

    /// `charge_root`, refused with the count it would have reached and the
    /// count recovery admits.
    pub(super) fn charge_root_with_evidence(&mut self) -> Result<RootUnit, (u64, u64)> {
        self.consume_with_evidence(1).map(|()| RootUnit(()))
    }

    /// A decoder stopped at the entries this budget had left, having counted
    /// `local_observed` of its own.
    pub(super) fn refuse_decoded(&mut self, local_observed: u64) -> PageObservationFailure {
        let (observed, _) = self.crossing_evidence(local_observed);
        self.refuse(observed)
    }

    /// One view holds `entries`, more than recovery admits in all.
    pub(super) fn refuse_view(&mut self, entries: u64) -> PageObservationFailure {
        self.refuse(entries)
    }

    fn refuse(&mut self, observed: u64) -> PageObservationFailure {
        self.refused_at = Some(observed);
        PageObservationFailure::ManifestEntryLimit
    }

    /// The count the latest refused charge would have reached. Observation
    /// stops there, so the whole need can be higher.
    pub(super) const fn refused_at(&self) -> Option<u64> {
        self.refused_at
    }

    /// Every entry recovery admits, whatever has been charged against it.
    pub(super) const fn admitted(&self) -> u64 {
        self.admitted
    }

    pub(super) const fn remaining(&self) -> u64 {
        self.admitted.saturating_sub(self.observed)
    }

    pub(super) const fn crossing_evidence(&self, local_observed: u64) -> (u64, u64) {
        (self.observed.saturating_add(local_observed), self.admitted)
    }

    pub(super) fn consume_with_evidence(&mut self, entries: usize) -> Result<(), (u64, u64)> {
        let observed = self.observed.saturating_add(entries as u64);
        if observed > self.admitted {
            return Err((observed, self.admitted));
        }
        self.observed = observed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ManifestEntryBudget, PageObservationFailure};

    #[test]
    fn a_refused_charge_records_the_count_it_would_have_reached() {
        let mut budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(budget.consume(3), Ok(()));
        assert_eq!(budget.refused_at(), None);
        assert_eq!(
            budget.consume(1),
            Err(PageObservationFailure::ManifestEntryLimit)
        );
        assert_eq!(budget.refused_at(), Some(11));

        let mut budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(
            budget.consume(5),
            Err(PageObservationFailure::ManifestEntryLimit)
        );
        assert_eq!(budget.refused_at(), Some(12));
        assert_eq!(
            budget.refuse_decoded(4),
            PageObservationFailure::ManifestEntryLimit
        );
        assert_eq!(budget.refused_at(), Some(11));
        assert_eq!(
            budget.refuse_view(13),
            PageObservationFailure::ManifestEntryLimit
        );
        assert_eq!(budget.refused_at(), Some(13));
    }

    #[test]
    fn a_root_is_charged_one_entry_and_refused_when_none_is_left() {
        let mut budget = ManifestEntryBudget::new(10, 8);
        assert!(budget.charge_root().is_ok());
        assert_eq!(budget.remaining(), 1);
        assert!(budget.charge_root_with_evidence().is_ok());
        assert_eq!(budget.remaining(), 0);
        assert_eq!(
            budget.charge_root().err(),
            Some(PageObservationFailure::ManifestEntryLimit)
        );
        assert_eq!(budget.refused_at(), Some(11));
        assert_eq!(budget.charge_root_with_evidence().err(), Some((11, 10)));
    }

    #[test]
    fn local_decoder_crossing_is_reported_in_global_coordinates() {
        let budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(budget.crossing_evidence(4), (11, 10));
    }
}
