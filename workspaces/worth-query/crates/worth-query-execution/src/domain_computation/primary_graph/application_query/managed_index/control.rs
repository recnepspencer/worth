//! Borrowed original checkpoints and first refusal; no replacement request or budget.
use super::WorthQueryManagedIndexStop as Stop;
use std::cell::RefCell;
use worth_execution::{MapKernelStop, WorkCeilingDenial};

/// The original request controls borrowed by a pure provider invocation.
pub struct WorthQueryManagedIndexControl<'a> {
    checkpoint: &'a dyn Fn(
        u64,
        Option<&worth_relational::facade::runtime::CustomInvariantReadBudget<'_>>,
    ) -> Result<(), Stop>,
    poll: &'a dyn Fn() -> Result<(), MapKernelStop>,
    reserve: &'a dyn Fn(u64) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop>,
    first: &'a RefCell<Option<Stop>>,
    source: Option<&'a worth_relational::facade::runtime::CustomInvariantReadBudget<'a>>,
}

impl<'a> WorthQueryManagedIndexControl<'a> {
    pub(super) fn new(
        checkpoint: &'a dyn Fn(
            u64,
            Option<&worth_relational::facade::runtime::CustomInvariantReadBudget<'_>>,
        ) -> Result<(), Stop>,
        poll: &'a dyn Fn() -> Result<(), MapKernelStop>,
        first: &'a RefCell<Option<Stop>>,
        reserve: &'a dyn Fn(
            u64,
        )
            -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop>,
    ) -> Self {
        Self {
            checkpoint,
            poll,
            reserve,
            first,
            source: None,
        }
    }

    pub fn checkpoint(&self, work: u64) -> Result<(), Stop> {
        self.check()?;
        (self.checkpoint)(work, self.source).map_err(|cause| self.refuse(cause))
    }

    pub fn poll(&self) -> Result<(), Stop> {
        self.check()?;
        if let Some(source) = self.source {
            source
                .check_live()
                .map_err(|cause| self.refuse(Stop::NativeSource(cause)))?;
        }
        (self.poll)().map_err(|cause| self.refuse(Stop::Request(WorkCeilingDenial::Stopped(cause))))
    }
    pub(super) fn under_source<'borrow>(
        &'borrow self,
        source: &'borrow worth_relational::facade::runtime::CustomInvariantReadBudget<'borrow>,
    ) -> WorthQueryManagedIndexControl<'borrow> {
        WorthQueryManagedIndexControl {
            checkpoint: self.checkpoint,
            poll: self.poll,
            reserve: self.reserve,
            first: self.first,
            source: Some(source),
        }
    }

    pub(super) fn reserve_content_memory(
        &self,
        bytes: u64,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop> {
        self.poll()?;
        (self.reserve)(bytes).map_err(|cause| self.refuse(cause))
    }

    pub(super) fn check(&self) -> Result<(), Stop> {
        match self.first.borrow().as_ref() {
            Some(cause) => Err(cause.clone()),
            None => Ok(()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn refuse(&self, cause: Stop) -> Stop {
        self.first.borrow_mut().get_or_insert(cause).clone()
    }
}

mod court_doors_for_test;
