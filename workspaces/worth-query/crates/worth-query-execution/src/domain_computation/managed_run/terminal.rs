use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind;



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
