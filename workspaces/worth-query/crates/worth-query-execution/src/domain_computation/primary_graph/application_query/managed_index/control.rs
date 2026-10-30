//! Borrowed original checkpoints and first refusal; no replacement request or budget.
use super::WorthQueryManagedIndexStop as Stop;
use std::cell::RefCell;
use worth_execution::{MapKernelStop, WorkCeilingDenial};

/// The original request controls borrowed by a pure provider invocation.
pub struct WorthQueryManagedIndexControl<'a> {
    checkpoint: &'a dyn Fn(u64) -> Result<(), MapKernelStop>,
    poll: &'a dyn Fn() -> Result<(), MapKernelStop>,
    first: &'a RefCell<Option<Stop>>,
}

impl<'a> WorthQueryManagedIndexControl<'a> {
    pub(super) fn new(
        checkpoint: &'a dyn Fn(u64) -> Result<(), MapKernelStop>,
        poll: &'a dyn Fn() -> Result<(), MapKernelStop>,
        first: &'a RefCell<Option<Stop>>,
    ) -> Self {
        Self {
            checkpoint,
            poll,
            first,
        }
    }

    pub fn checkpoint(&self, work: u64) -> Result<(), Stop> {
        self.check()?;
        (self.checkpoint)(work)
            .map_err(|cause| self.refuse(Stop::Request(WorkCeilingDenial::Stopped(cause))))
    }

    pub fn poll(&self) -> Result<(), Stop> {
        self.check()?;
        (self.poll)().map_err(|cause| self.refuse(Stop::Request(WorkCeilingDenial::Stopped(cause))))
    }

    pub(super) fn check(&self) -> Result<(), Stop> {
        match self.first.borrow().as_ref() {
            Some(cause) => Err(cause.clone()),
            None => Ok(()),
        }
    }

    pub(super) fn refuse(&self, cause: Stop) -> Stop {
        self.first.borrow_mut().get_or_insert(cause).clone()
    }
}
