#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedStoragePreparationDenial {
    WorkExhausted { maximum_visits: usize },
    ChargeOverflow,
    ChargeUnderflow,
    RetainedExtentHistoryUnavailable,
}

/// One explicit bounded traversal. No charge becomes usable from a partial
/// result, and a failed traversal cannot replenish its own work allowance.
pub(crate) struct RetainedStoragePreparation {
    maximum_visits: usize,
    visits: usize,
}

impl RetainedStoragePreparation {
    /// Temporarily cap additional work without creating or resetting a counter.
    pub(crate) fn limit_additional_visits(
        &mut self,
        maximum_additional: usize,
    ) -> RetainedStoragePreparationLimit<'_> {
        RetainedStoragePreparationLimit::new(self, maximum_additional)
    }

    pub(crate) const fn new(maximum_visits: usize) -> Self {
        Self {
            maximum_visits,
            visits: 0,
        }
    }

    pub(crate) const fn visits(&self) -> usize {
        self.visits
    }

    pub(crate) const fn maximum_visits(&self) -> usize {
        self.maximum_visits
    }

    pub(crate) fn visit(&mut self) -> Result<(), RetainedStoragePreparationDenial> {
        self.reserve_visits(1)
    }

    /// Admit a conservative traversal bound before entering an opaque storage
    /// iterator. Failure preserves consumed work and exposes no partial charge.
    pub(crate) fn reserve_visits(
        &mut self,
        count: usize,
    ) -> Result<(), RetainedStoragePreparationDenial> {
        let visits = self
            .visits
            .checked_add(count)
            .filter(|visits| *visits <= self.maximum_visits)
            .ok_or(RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: self.maximum_visits,
            })?;
        self.visits = visits;
        Ok(())
    }
}
mod limit;
pub(crate) use limit::RetainedStoragePreparationLimit;
