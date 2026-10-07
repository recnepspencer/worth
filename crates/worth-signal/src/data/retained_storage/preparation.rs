#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedStoragePreparationDenial {
    WorkExhausted { maximum_visits: usize },
    ChargeOverflow,
    ChargeUnderflow,
    RetainedExtentHistoryUnavailable,
}

/// One explicit bounded traversal. No charge becomes usable from a partial
/// result, and a failed traversal cannot replenish its own work allowance.
pub(crate) struct RetainedStoragePreparation<'observer> {
    maximum_visits: usize,
    visits: usize,
    checkpoint:
        Option<&'observer mut dyn FnMut(usize) -> Result<(), RetainedStoragePreparationDenial>>,
}

impl RetainedStoragePreparation<'static> {
    pub(crate) const fn new(maximum_visits: usize) -> Self {
        Self {
            maximum_visits,
            visits: 0,
            checkpoint: None,
        }
    }
}

impl<'observer> RetainedStoragePreparation<'observer> {
    /// Temporarily cap additional work without creating or resetting a counter.
    pub(crate) fn limit_additional_visits(
        &mut self,
        maximum_additional: usize,
    ) -> RetainedStoragePreparationLimit<'_, 'observer> {
        RetainedStoragePreparationLimit::new(self, maximum_additional)
    }

    /// Lend this same monotonic counter to one request-checkpoint scope.
    /// The guard returns every consumed visit on drop, including unwind.
    pub(crate) fn reborrow_with_checkpoint<'work, 'checkpoint>(
        &'work mut self,
        checkpoint: &'checkpoint mut dyn FnMut(
            usize,
        )
            -> Result<(), RetainedStoragePreparationDenial>,
    ) -> RetainedStoragePreparationCheckpoint<'work, 'checkpoint, 'observer> {
        let maximum_visits = self.maximum_visits;
        let visits = self.visits;
        RetainedStoragePreparationCheckpoint {
            owner: self,
            scoped: RetainedStoragePreparation {
                maximum_visits,
                visits,
                checkpoint: Some(checkpoint),
            },
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
        self.checkpoint_request_only(count)?;
        self.visits = visits;
        Ok(())
    }

    /// Check request-owned work that is outside the retained policy's visit
    /// axis (for example bounded semantic projection bytes). This never
    /// changes the local visit counter or creates independent authority.
    pub(crate) fn checkpoint_request_only(
        &mut self,
        units: usize,
    ) -> Result<(), RetainedStoragePreparationDenial> {
        if let Some(checkpoint) = self.checkpoint.as_deref_mut() {
            checkpoint(units)?;
        }
        Ok(())
    }
}
mod limit;
pub(crate) use limit::{RetainedStoragePreparationCheckpoint, RetainedStoragePreparationLimit};
