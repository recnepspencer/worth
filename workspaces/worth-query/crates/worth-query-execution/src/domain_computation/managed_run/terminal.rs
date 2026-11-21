pub use worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::{
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
    WorthQueryManagedRunTerminalKind,
};

pub(in crate::domain_computation::managed_run) fn bridge_terminal_disposition(
    terminal: WorthQueryManagedRunTerminalKind,
) -> worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition {
    use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

    match terminal {
        WorthQueryManagedRunTerminalKind::Completed => {
            BridgeExecutionBasisTerminalDisposition::Completed
        }
        WorthQueryManagedRunTerminalKind::Cancelled => {
            BridgeExecutionBasisTerminalDisposition::Cancelled
        }
        WorthQueryManagedRunTerminalKind::TimedOut
        | WorthQueryManagedRunTerminalKind::Exhausted
        | WorthQueryManagedRunTerminalKind::Degraded
        | WorthQueryManagedRunTerminalKind::Failed => {
            BridgeExecutionBasisTerminalDisposition::Abandoned
        }
    }
}
#[cfg(test)]
#[path = "terminal_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
