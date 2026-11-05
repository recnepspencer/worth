use crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunDenialKind {
    ForeignQueryRuntime,
    StaleInstallationGeneration,
    ResourceAttemptMismatch,
    BridgeManagedIntentMismatch,
    MissingBridgeSourceAuthority,
    ForeignRelationalRuntime,
    NonRelationalSnapshotIdentity,
    RelationalSnapshotMismatch,
    SemanticBasisMismatch,
    SemanticBasisUnsupported,
    UnverifiedProviderWork,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedRunDenial {
    kind: WorthQueryManagedRunDenialKind,
    detail: &'static str,
    counters: WorthQueryManagedRunCounters,
}

impl WorthQueryManagedRunDenial {
    pub(crate) fn new(
        kind: WorthQueryManagedRunDenialKind,
        detail: &'static str,
        counters: WorthQueryManagedRunCounters,
    ) -> Self {
        Self {
            kind,
            detail,
            counters,
        }
    }

    pub fn kind(&self) -> WorthQueryManagedRunDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &'static str {
        self.detail
    }

    pub fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }
}
