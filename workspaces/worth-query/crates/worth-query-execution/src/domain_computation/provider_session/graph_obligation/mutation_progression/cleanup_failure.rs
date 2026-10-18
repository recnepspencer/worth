//! Exact managed cleanup custody, distinct from inconsistent completion evidence.
use super::WorthQueryMutationRunBinding;
use crate::domain_computation::{
    WorthQueryDirectRunCleanupFailure, WorthQueryDirectRunCleanupReceipt,
    WorthQueryRunningDirectRun,
};

/// Native cleanup refusal is distinct from an inconsistent binding or completion witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMutationCleanupFailureKind {
    Native(crate::domain_computation::WorthQueryManagedRunCleanupFailureKind),
    AffinityInconsistent,
    CompletionInconsistent,
}

/// A refused mutation cleanup retains its native owner and exact graph-work binding.
#[must_use = "retain failed native cleanup until lawful discharge"]
pub struct WorthQueryMutationCleanupFailure {
    binding: WorthQueryMutationRunBinding,
    provider_session: Option<
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    >,
    custody: Custody,
    snapshot_released: bool,
}
enum Custody {
    Native(WorthQueryDirectRunCleanupFailure),
    Affinity(WorthQueryRunningDirectRun),
    Completion(WorthQueryDirectRunCleanupReceipt),
}
impl WorthQueryMutationCleanupFailure {
    pub(super) fn native(
        binding: WorthQueryMutationRunBinding,
        failure: WorthQueryDirectRunCleanupFailure,
        snapshot_released: bool,
    ) -> Self {
        Self {
            binding,
            provider_session: None,
            custody: Custody::Native(failure),
            snapshot_released,
        }
    }
    pub(super) fn affinity(
        binding: WorthQueryMutationRunBinding,
        running: WorthQueryRunningDirectRun,
    ) -> Self {
        Self {
            binding,
            provider_session: None,
            custody: Custody::Affinity(running),
            snapshot_released: false,
        }
    }
    pub(super) fn completion(
        binding: WorthQueryMutationRunBinding,
        receipt: WorthQueryDirectRunCleanupReceipt,
    ) -> Self {
        Self {
            binding,
            provider_session: None,
            custody: Custody::Completion(receipt),
            snapshot_released: false,
        }
    }
    pub(super) fn with_provider_session(
        mut self,
        session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        self.provider_session = Some(session);
        self
    }
    pub(in crate::domain_computation) fn retry(
        self,
    ) -> Result<WorthQueryDirectRunCleanupReceipt, Self> {
        let Self {
            binding,
            provider_session,
            custody,
            snapshot_released,
        } = self;
        let Custody::Native(failure) = custody else {
            return Err(Self {
                binding,
                provider_session,
                custody,
                snapshot_released,
            });
        };
        match failure.retry() {
            Err(failure) => Err(Self {
                binding,
                provider_session,
                custody: Custody::Native(failure),
                snapshot_released,
            }),
            Ok(receipt) => {
                let inspection = receipt.inspection();
                if snapshot_released
                    && inspection.resources_released()
                    && inspection.run_identity() == binding.worker.as_ref()
                    && inspection.provider_session_identity() == binding.provider_session.as_ref()
                    && inspection.resource_plan_identity() == binding.resource_plan.as_ref()
                    && inspection.released_reservation_count() == binding.reservation_count
                    && inspection.provider_work().provider_retained_bytes() == 0
                {
                    Ok(receipt)
                } else {
                    Err(Self {
                        binding,
                        provider_session,
                        custody: Custody::Completion(receipt),
                        snapshot_released,
                    })
                }
            }
        }
    }
    /// The actual native refusal or non-retryable consistency failure.
    pub fn kind(&self) -> WorthQueryMutationCleanupFailureKind {
        match &self.custody {
            Custody::Native(failure) => {
                WorthQueryMutationCleanupFailureKind::Native(failure.failure_kind())
            }
            Custody::Affinity(_) => WorthQueryMutationCleanupFailureKind::AffinityInconsistent,
            Custody::Completion(_) => WorthQueryMutationCleanupFailureKind::CompletionInconsistent,
        }
    }
    /// The retained native run identity, including the actual run on affinity mismatch.
    pub fn run_identity(&self) -> &str {
        match &self.custody {
            Custody::Native(_) => self.binding.worker.as_ref(),
            Custody::Affinity(running) => running.identity(),
            Custody::Completion(receipt) => receipt.inspection().run_identity(),
        }
    }
    /// Original admitted graph-work session identity; no fresh session is inferred.
    pub fn session_identity(&self) -> super::WorthQueryGraphWorkSessionIdentity {
        self.binding.session
    }
    /// Original admitted managed-run identity retained with cleanup custody.
    pub fn managed_run_identity(&self) -> super::WorthQueryGraphWorkManagedRunIdentity {
        self.binding.managed_run
    }
}
impl std::fmt::Debug for WorthQueryMutationCleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryMutationCleanupFailure")
            .field("kind", &self.kind())
            .field("session", &self.binding.session)
            .finish_non_exhaustive()
    }
}
