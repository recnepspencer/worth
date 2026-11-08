//! Admission stays at path orchestration; cell constructors receive owned bytes only.
use super::ExecutionOrderedMapDenial as Denial;
use crate::{ExecutionAllocationPolicy, ExecutionMemoryReservation};
use std::cell::RefCell;

pub(super) trait FundingSource {
    fn uses_accounting(&self) -> bool;
    fn reserve_memory(&self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Denial>;
}
impl FundingSource for ExecutionAllocationPolicy<'_, '_> {
    fn uses_accounting(&self) -> bool {
        (*self).uses_accounting()
    }
    fn reserve_memory(&self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Denial> {
        (*self).reserve_memory(bytes).map_err(Denial::Allocation)
    }
}

pub(super) struct PathFunding {
    accounted: bool,
    owned: RefCell<Option<ExecutionMemoryReservation>>,
}
impl PathFunding {
    pub(super) fn admit(source: &dyn FundingSource, bytes: u64) -> Result<Self, Denial> {
        let accounted = source.uses_accounting();
        let owned = if accounted {
            source.reserve_memory(bytes)?
        } else {
            None
        };
        Ok(Self {
            accounted,
            owned: RefCell::new(owned),
        })
    }
    pub(super) fn uses_accounting(&self) -> bool {
        self.accounted
    }
    pub(super) fn reserve_memory(
        &self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, Denial> {
        let mut owned = self.owned.borrow_mut();
        match owned.as_mut() {
            Some(owned) => owned
                .split_untracked(bytes)
                .map(Some)
                .ok_or(Denial::CapacityOverflow),
            // Explicit system allocation retains accounting metadata without
            // inventing a request grant. No source or request exists here.
            None => Ok(None),
        }
    }
}
