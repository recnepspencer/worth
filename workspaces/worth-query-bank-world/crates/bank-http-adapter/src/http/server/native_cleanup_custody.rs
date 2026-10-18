//! Original uncommitted business evidence survives physical release and host handoff.
use bank_server::{BankManagedCleanup, BankReleasedManagedCleanup};
pub(super) enum NativeCleanupCustody {
    Pending(BankManagedCleanup),
    Released(BankReleasedManagedCleanup),
}
pub(super) struct NativeCleanupRecord {
    pub(super) owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
    pub(super) sequence: u64,
    pub(super) custody: NativeCleanupCustody,
    pub(super) permit: tokio::sync::OwnedSemaphorePermit,
}
/// Move-only host evidence after lawful native physical release. This is not a commit.
#[must_use = "retain original business evidence and the exact installed runtime"]
pub struct BankHttpReleasedCleanup {
    pub(super) runtime: std::sync::Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
    pub(super) evidence: BankReleasedManagedCleanup,
}
impl BankHttpReleasedCleanup {
    /// Borrows the exact installed Bank allocation retained by this handoff.
    pub fn runtime(&self) -> &bank_server::BankIdentityRuntime {
        self.runtime.runtime()
    }
    /// Original uncommitted evidence and native physical receipt remain distinct.
    pub fn evidence(&self) -> &BankReleasedManagedCleanup {
        &self.evidence
    }
    /// Transfers original evidence with its existing runtime allocation kept alive.
    pub fn into_parts(self) -> (BankHttpCleanupRuntime, BankReleasedManagedCleanup) {
        (
            BankHttpCleanupRuntime {
                application: self.runtime,
            },
            self.evidence,
        )
    }
}

/// Keeps the existing installed allocation alive for fresh native inspection and recovery.
/// It installs nothing and grants no business permission or process durability.
#[must_use = "native discovery and cleanup require the original runtime to remain alive"]
pub struct BankHttpCleanupRuntime {
    application: std::sync::Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
}
impl BankHttpCleanupRuntime {
    /// The original Bank runtime, including its lawful request and cleanup entrances.
    pub fn runtime(&self) -> &bank_server::BankIdentityRuntime {
        self.application.runtime()
    }
}

pub(super) struct HandoffRecord {
    pub(super) permit: tokio::sync::OwnedSemaphorePermit,
}
