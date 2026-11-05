use super::super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;
use {
    super::super::provider_work::WorthQueryManagedProviderCleanupAuthority,
    super::super::run_affinity::WorthQueryDirectRunTerminalAffinity,
    crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters,
    crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence,
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind,
};
use {
    super::cleanup, super::WorthQueryDirectRunCleanupFailure,
    crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupReceipt,
};

#[must_use = "a direct run terminal retains resources that must be cleaned up"]
pub struct WorthQueryDirectRunTerminal {
    pub(in crate::domain_computation::managed_run) affinity: WorthQueryDirectRunTerminalAffinity,
    pub(in crate::domain_computation::managed_run) kind: WorthQueryManagedRunTerminalKind,
    pub(in crate::domain_computation::managed_run) bridge_basis: BridgeBoundExecutionBasis,
    pub(in crate::domain_computation::managed_run) relational_basis:
        WorthQueryManagedRelationalObservation,
    pub(in crate::domain_computation::managed_run) counters: WorthQueryManagedRunCounters,
    pub(in crate::domain_computation::managed_run) provider_work:
        WorthQueryManagedProviderWorkEvidence,
    pub(in crate::domain_computation::managed_run) provider_cleanup:
        WorthQueryManagedProviderCleanupAuthority,
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
