use std::sync::Arc;

use crate::domain_computation::managed_run::bridge_terminal_disposition;
use crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryCompletedDirectRunCleanup;
use {
    super::WorthQueryDirectRunTerminal,
    crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupReceipt,
};
use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition,
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupFailureKind,
};

#[must_use = "direct cleanup failure retains the exact terminal owner for retry"]
pub struct WorthQueryDirectRunCleanupFailure {
    failure_kind: WorthQueryManagedRunCleanupFailureKind,
    failure_detail: Arc<str>,
    terminal: WorthQueryDirectRunTerminal,
}

pub(super) fn finish(
    terminal: WorthQueryDirectRunTerminal,
) -> Result<WorthQueryDirectRunCleanupReceipt, WorthQueryDirectRunCleanupFailure> {
    let terminal = release_provider_queue_occupancy(terminal)?;
    let terminal = reconcile_provider_memory(terminal)?;
    finalize_terminal(terminal)
}

fn release_provider_queue_occupancy(
    mut terminal: WorthQueryDirectRunTerminal,
) -> Result<WorthQueryDirectRunTerminal, WorthQueryDirectRunCleanupFailure> {
    if let Err(failure) = terminal
        .provider_cleanup
        .release_queue_occupancies(&mut terminal.bridge_basis, &mut terminal.provider_work)
    {
        return Err(WorthQueryDirectRunCleanupFailure {
            failure_kind: WorthQueryManagedRunCleanupFailureKind::QueueRelease(failure.kind()),
            failure_detail: Arc::from(failure.detail()),
            terminal,
        });
    }
    Ok(terminal)
}

fn reconcile_provider_memory(
    mut terminal: WorthQueryDirectRunTerminal,
) -> Result<WorthQueryDirectRunTerminal, WorthQueryDirectRunCleanupFailure> {
    let retained_bytes = terminal.provider_cleanup.reconcile_provider_memory();
    terminal
        .provider_work
        .reconcile_provider_retained_bytes(retained_bytes);
    if retained_bytes == 0 {
        return Ok(terminal);
    }
    Err(WorthQueryDirectRunCleanupFailure {
        failure_kind: WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained,
        failure_detail: Arc::from(format!(
            "managed provider retains {retained_bytes} governed bytes"
        )),
        terminal,
    })
}

fn finalize_terminal(
    terminal: WorthQueryDirectRunTerminal,
) -> Result<WorthQueryDirectRunCleanupReceipt, WorthQueryDirectRunCleanupFailure> {
    let WorthQueryDirectRunTerminal {
        affinity,
        kind,
        bridge_basis,
        relational_basis,
        counters,
        provider_work,
        provider_cleanup,
    } = terminal;
    let bridge = match bridge_basis.finalize(bridge_terminal_disposition(kind)) {
        Ok(receipt) => receipt,
        Err(failure) => {
            return Err(WorthQueryDirectRunCleanupFailure {
                failure_kind: WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
                    failure.kind(),
                ),
                failure_detail: Arc::from(failure.detail()),
                terminal: WorthQueryDirectRunTerminal {
                    affinity,
                    kind,
                    bridge_basis: failure.into_basis(),
                    relational_basis,
                    counters,
                    provider_work,
                    provider_cleanup,
                },
            });
        }
    };
    let (logical_run_identity, identity) = affinity.terminal_descriptions();
    let disposition = if provider_work.requires_cleanup_recovery() {
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    } else {
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    };
    Ok(WorthQueryDirectRunCleanupReceipt::from_completed(
        WorthQueryCompletedDirectRunCleanup {
            logical_run_identity,
            identity,
            terminal: kind,
            disposition,
            bridge,
            relational: relational_basis.release(),
            attempt: affinity.release(),
            counters,
            provider_work,
        },
    ))
}

impl WorthQueryDirectRunCleanupFailure {
    pub fn failure_kind(&self) -> WorthQueryManagedRunCleanupFailureKind {
        self.failure_kind
    }

    pub fn failure_detail(&self) -> &str {
        &self.failure_detail
    }

    pub fn provider_retained_bytes(&self) -> usize {
        self.terminal.provider_work.provider_retained_bytes()
    }

    pub fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        match self.failure_kind {
            WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained => {
                WorthQueryManagedRunCleanupDisposition::CleanupPending
            }
            WorthQueryManagedRunCleanupFailureKind::QueueRelease(_)
            | WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(_) => {
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired
            }
        }
    }

    #[must_use = "retry returns the next cleanup result and must be resolved"]
    pub fn retry(
        self,
    ) -> Result<WorthQueryDirectRunCleanupReceipt, WorthQueryDirectRunCleanupFailure> {
        self.terminal.cleanup()
    }

    #[must_use = "the returned terminal retains the exact cleanup authority"]
    pub fn into_terminal(self) -> WorthQueryDirectRunTerminal {
        self.terminal
    }
}

impl std::fmt::Debug for WorthQueryDirectRunCleanupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDirectRunCleanupFailure")
            .field("failure_kind", &self.failure_kind)
            .field("failure_detail", &self.failure_detail)
            .field("run_identity", &self.terminal.identity())
            .finish()
    }
}
