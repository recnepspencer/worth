//! Unexported crate-boundary port exposing owner-private `managed_run_custody`
//! on `WorthQueryExecutionRuntime` strictly to upper execution split crates.

use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::managed_run::durable_custody::ManagedRunCustodyRegistry;

pub trait ExecutionRuntimeManagedRunCustodyPort {
    fn managed_run_custody(&self) -> &ManagedRunCustodyRegistry;
}

impl ExecutionRuntimeManagedRunCustodyPort for WorthQueryExecutionRuntime {
    fn managed_run_custody(&self) -> &ManagedRunCustodyRegistry {
        &self.managed_run_custody
    }
}
