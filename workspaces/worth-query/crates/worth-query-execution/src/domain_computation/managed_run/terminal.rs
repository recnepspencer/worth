#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunTerminalKind {
    Completed,
    Cancelled,
    TimedOut,
    Exhausted,
    Degraded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunCleanupDisposition {
    CleanupComplete,
    CleanupPending,
    RecoveryRequired,
}

/// The native managed-run owner that refused cleanup; the typed failure retains its exact terminal custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunCleanupFailureKind {
    ProviderMemoryRetained,
    QueueRelease(worth_runtime_bridge::facade::BridgeManagedQueueFailureKind),
    BridgeFinalization(worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationFailureKind),
}

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
