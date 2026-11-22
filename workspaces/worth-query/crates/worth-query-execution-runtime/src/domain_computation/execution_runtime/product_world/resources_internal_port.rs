//! Crate-boundary internal port for `WorthQueryProductWorldResources`.
//! Never exported on `worth-query-execution::facade` or Host.

use worth_runtime_world::facade::RuntimeWorldBudgets;

use super::resources::{WorthQueryProductExecution, WorthQueryProductWorldResources};
use super::WorthQueryProductWorldClock;

pub trait ProductWorldResourcesInternalPort: Sized {
    fn admits_clock(
        &self,
        clock: &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> bool;

    fn into_parts(
        self,
    ) -> (
        RuntimeWorldBudgets,
        WorthQueryProductWorldClock,
        super::super::WorthQueryInvalidationResources,
        WorthQueryProductExecution,
    );
}

impl ProductWorldResourcesInternalPort for WorthQueryProductWorldResources {
    fn admits_clock(
        &self,
        clock: &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> bool {
        Self::admits_clock(self, clock)
    }

    fn into_parts(
        self,
    ) -> (
        RuntimeWorldBudgets,
        WorthQueryProductWorldClock,
        super::super::WorthQueryInvalidationResources,
        WorthQueryProductExecution,
    ) {
        Self::into_parts(self)
    }
}
