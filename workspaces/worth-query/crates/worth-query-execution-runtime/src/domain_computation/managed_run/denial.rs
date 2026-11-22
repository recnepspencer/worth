use super::WorthQueryManagedRunCounters;

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
    pub fn new(
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
