//! Moved whole from `domain_computation::managed_run::terminal`: the items below and every impl they own.

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
