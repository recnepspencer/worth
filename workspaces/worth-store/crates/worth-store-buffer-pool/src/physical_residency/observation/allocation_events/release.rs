//! Preflight one physical release's complete evidence before its owner effects.
use super::{
    PhysicalResidencyAllocationBoundaryFacts as Facts,
    PhysicalResidencyAllocationEventCells as Cells,
    PhysicalResidencyAllocationEventCounters as Counters,
    PhysicalResidencyAllocationEventRecorder as Recorder, PreparedTracePage,
};
use crate::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
    PhysicalResidencyRetirementDenial as Denial,
};
use std::sync::{MutexGuard, TryLockError};

pub(crate) struct PreparedReleaseEvents<'recorder> {
    cells: MutexGuard<'recorder, Cells>,
    counters: [Counters; Dimension::COUNT],
    facts: [Option<Facts>; 3],
}

impl Recorder {
    pub(crate) fn prepare_pin_release(
        &self,
        last_pin: bool,
        page: &mut PreparedTracePage,
    ) -> Result<PreparedReleaseEvents<'_>, Denial> {
        self.prepare_releases(
            [
                Some(Facts::release(Dimension::PinLeases, None, 1)),
                last_pin.then(|| Facts::release(Dimension::PinnedFrames, None, 1)),
                None,
            ],
            page,
        )
    }

    pub(crate) fn prepare_operation_release(
        &self,
        scope: Scope,
        bytes: u64,
        page: &mut PreparedTracePage,
    ) -> Result<PreparedReleaseEvents<'_>, Denial> {
        let release =
            |dimension| (bytes != 0).then(|| Facts::release(dimension, Some(scope), bytes));
        self.prepare_releases(
            [
                release(Dimension::OperationBytes),
                release(Dimension::OperationScope(scope)),
                release(Dimension::TotalBytes),
            ],
            page,
        )
    }

    fn prepare_releases(
        &self,
        facts: [Option<Facts>; 3],
        page: &mut PreparedTracePage,
    ) -> Result<PreparedReleaseEvents<'_>, Denial> {
        let cells = self.cells.try_lock().map_err(|cause| match cause {
            TryLockError::WouldBlock => Denial::Busy,
            TryLockError::Poisoned(_) => Denial::PoisonedOwner,
        })?;
        #[cfg(feature = "certification-test-authority")]
        let mut cells = cells;
        let mut counters = cells.dimensions;
        for fact in facts.iter().flatten() {
            let counter = &mut counters[fact.dimension.index()];
            counter.releases = counter
                .releases
                .checked_add(1)
                .ok_or(Denial::AccountingMismatch)?;
            counter.released_units = counter
                .released_units
                .checked_add(fact.actual_units)
                .filter(|released| *released <= counter.admitted_units)
                .ok_or(Denial::AccountingMismatch)?;
        }
        #[cfg(feature = "certification-test-authority")]
        {
            let count = facts.iter().flatten().count();
            cells
                .next_sequence
                .checked_add(count as u64)
                .ok_or(Denial::EventSequenceExhausted)?;
            if !cells.events.can_extend(count) {
                return Err(Denial::EventSequenceExhausted);
            }
            cells.events.prepare_capacity(count, page);
        }
        #[cfg(not(feature = "certification-test-authority"))]
        let _ = page;
        Ok(PreparedReleaseEvents {
            cells,
            counters,
            facts,
        })
    }
}

impl PreparedReleaseEvents<'_> {
    /// Infallible after preparation; the SAME recorder gate remains held.
    pub(crate) fn commit(mut self) {
        self.cells.dimensions = self.counters;
        for fact in self.facts.into_iter().flatten() {
            #[cfg(feature = "certification-test-authority")]
            {
                let sequence = self.cells.next_sequence;
                self.cells.next_sequence += 1;
                self.cells.events.push_prepared(
                    super::PhysicalResidencyAllocationBoundaryEvent::new(sequence, fact),
                );
            }
            #[cfg(not(feature = "certification-test-authority"))]
            let _ = fact;
        }
    }
}
