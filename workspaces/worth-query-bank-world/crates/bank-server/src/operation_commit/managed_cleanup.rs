//! Move-only native cleanup and business evidence; the native issuer decides release.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_contribution::{WorthQueryAdvancementDenial, WorthQueryAdvancementPhase},
    primary_graph::{
        WorthQueryApplicationCleanupRequired, WorthQueryApplicationManagedCleanup,
        WorthQueryApplicationUncommitted, WorthQueryMutationCleanupFailure,
    },
    runtime::WorthQueryDirectRunCleanupReceipt,
};

/// Actual failed native cleanup. Neither a clone nor a business completion.
/// ```compile_fail,E0599
/// fn no_clone(owner: bank_server::BankManagedCleanup) { let _ = owner.clone(); }
/// ```
#[derive(Debug)]
#[must_use = "retain actual cleanup and original business evidence"]
pub struct BankManagedCleanup {
    business: WorthQueryApplicationUncommitted,
    native: WorthQueryApplicationManagedCleanup,
}
/// Physical cleanup evidence remains separate from the original business outcome.
/// ```compile_fail,E0308
/// fn not_a_commit(owner: bank_server::BankReleasedManagedCleanup) -> bank_server::BankCommitReceipt {
///     owner.into_parts().1
/// }
/// ```
#[must_use = "release does not settle the original business outcome"]
pub struct BankReleasedManagedCleanup {
    business: WorthQueryApplicationUncommitted,
    receipt: WorthQueryDirectRunCleanupReceipt,
}
impl From<Box<WorthQueryApplicationCleanupRequired>> for BankManagedCleanup {
    fn from(required: Box<WorthQueryApplicationCleanupRequired>) -> Self {
        let (business, native) = required.into_parts();
        Self { business, native }
    }
}
impl BankManagedCleanup {
    /// Borrows the original native uncommitted outcome.
    pub fn business(&self) -> &WorthQueryApplicationUncommitted {
        &self.business
    }
    /// Borrows the native issuer or execution refusal from the latest retry.
    pub fn last_release_denial(&self) -> Option<&WorthQueryAdvancementDenial> {
        self.native.last_retry_denial()
    }
    /// Borrows the original native cleanup failure and run binding.
    pub fn failure(&self) -> &WorthQueryMutationCleanupFailure {
        self.native.failure()
    }
    /// Delegates physical release to the native owner under this admitted phase.
    pub fn release_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<BankReleasedManagedCleanup, Self> {
        let Self { business, native } = self;
        match native.retry_in_advancement(phase) {
            Ok(receipt) => Ok(BankReleasedManagedCleanup { business, receipt }),
            Err(native) => Err(Self { business, native }),
        }
    }
    /// Admission refusal never captures or drops this owner inside an unexecuted callback.
    pub fn release(
        self,
        runtime: &crate::BankIdentityRuntime,
        scope: &WorthQueryRequestScope,
    ) -> Result<BankReleasedManagedCleanup, (Self, Option<WorthQueryAdvancementDenial>)> {
        let mut owner = Some(self);
        match runtime.with_application_advancement(scope, |phase| {
            owner
                .take()
                .expect("admitted cleanup owns its original custody")
                .release_in_advancement(&phase)
        }) {
            Ok(Ok(receipt)) => Ok(receipt),
            Ok(Err(owner)) => Err((owner, None)),
            Err(denial) => Err((owner.expect("refusal precedes callback"), Some(denial))),
        }
    }
}
impl BankReleasedManagedCleanup {
    /// Borrows the original native uncommitted outcome.
    pub fn business(&self) -> &WorthQueryApplicationUncommitted {
        &self.business
    }
    /// Borrows the actual physical receipt; this is not a Bank commit.
    pub fn receipt(&self) -> &WorthQueryDirectRunCleanupReceipt {
        &self.receipt
    }
    /// Transfers actual evidence, not a reconstructed receipt or completion.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationUncommitted,
        WorthQueryDirectRunCleanupReceipt,
    ) {
        (self.business, self.receipt)
    }
}
