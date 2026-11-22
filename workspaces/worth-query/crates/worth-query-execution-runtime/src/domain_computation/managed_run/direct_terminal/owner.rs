use super::super::WorthQueryManagedRelationalObservation;
use super::super::{
    provider_work::WorthQueryManagedProviderCleanupAuthority,
    run_affinity::WorthQueryDirectRunTerminalAffinity, WorthQueryManagedProviderWorkEvidence,
    WorthQueryManagedRunCounters, WorthQueryManagedRunTerminalKind,
};
use super::{cleanup, WorthQueryDirectRunCleanupFailure, WorthQueryDirectRunCleanupReceipt};
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

#[must_use = "a direct run terminal retains resources that must be cleaned up"]
pub struct WorthQueryDirectRunTerminal {
    pub(crate) affinity: WorthQueryDirectRunTerminalAffinity,
    pub(crate) kind: WorthQueryManagedRunTerminalKind,
    pub(crate) bridge_basis: BridgeBoundExecutionBasis,
    pub(crate) relational_basis: WorthQueryManagedRelationalObservation,
    pub(crate) counters: WorthQueryManagedRunCounters,
    pub(crate) provider_work: WorthQueryManagedProviderWorkEvidence,
    pub(crate) provider_cleanup: WorthQueryManagedProviderCleanupAuthority,
}

impl WorthQueryDirectRunTerminal {
    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn kind(&self) -> WorthQueryManagedRunTerminalKind {
        self.kind
    }

    pub fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }

    pub fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }

    pub fn cleanup(
        self,
    ) -> Result<WorthQueryDirectRunCleanupReceipt, WorthQueryDirectRunCleanupFailure> {
        cleanup::finish(self)
    }
}
