use super::page_observation::PageObservationFailure;

pub(super) struct ManifestEntryBudget {
    admitted: u64,
    observed: u64,
}

impl ManifestEntryBudget {
    pub(super) const fn new(admitted: u64, already_observed: u64) -> Self {
        Self {
            admitted,
            observed: already_observed,
        }
    }

    pub(super) fn admit_pending_block_read(&self) -> Result<(), PageObservationFailure> {
        (self.remaining() != 0)
            .then_some(())
            .ok_or(PageObservationFailure::ManifestEntryLimit)
    }

    pub(super) fn consume(&mut self, entries: usize) -> Result<(), PageObservationFailure> {
        self.consume_with_evidence(entries)
            .map_err(|_| PageObservationFailure::ManifestEntryLimit)
    }

    pub(super) const fn remaining(&self) -> u64 {
        self.admitted.saturating_sub(self.observed)
    }

    pub(super) const fn crossing_evidence(&self, local_observed: u64) -> (u64, u64) {
        (self.observed.saturating_add(local_observed), self.admitted)
    }

    pub(super) const fn successor_read_evidence(&self) -> Result<(), (u64, u64)> {
        if self.remaining() == 0 {
            Err(self.crossing_evidence(1))
        } else {
            Ok(())
        }
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
    fn exhausted_branch_budget_denies_the_child_before_its_read() {
        let mut budget = ManifestEntryBudget::new(3, 2);
        assert_eq!(budget.admit_pending_block_read(), Ok(()));
        assert_eq!(budget.consume(1), Ok(()));
        assert_eq!(
            budget.admit_pending_block_read(),
            Err(PageObservationFailure::ManifestEntryLimit)
        );
    }

    #[test]
    fn local_decoder_crossing_is_reported_in_global_coordinates() {
        let budget = ManifestEntryBudget::new(10, 7);
        assert_eq!(budget.crossing_evidence(4), (11, 10));
    }
}
