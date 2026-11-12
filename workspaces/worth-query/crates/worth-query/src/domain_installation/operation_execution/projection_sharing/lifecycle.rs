use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::{WorthQueryRuntimeError, WorthQuerySharedLeaseRelease, WorthQueryWorkspace};

use super::super::WorthQuerySettledDomainProjection;
use super::WorthQuerySharedLiveProjectionLease;

pub struct WorthQueryDisposedSharedProjection<D, O, F, L: BasisOperationLane> {
    source: WorthQuerySettledDomainProjection<D, O, F, L>,
    release: WorthQuerySharedLeaseRelease,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryDisposedSharedProjection<D, O, F, L> {
    pub fn snapshot(&self) -> &WorthQuerySettledDomainProjection<D, O, F, L> {
        &self.source
    }

    pub fn release(&self) -> &WorthQuerySharedLeaseRelease {
        &self.release
    }
}

#[must_use = "failed lease disposal retains the exact active lease for retry"]
pub enum WorthQuerySharedProjectionDisposalOutcome<D, O, F, L: BasisOperationLane> {
    Disposed(WorthQueryDisposedSharedProjection<D, O, F, L>),
    Stopped(WorthQuerySharedProjectionDisposalStop<D, O, F, L>),
}

pub struct WorthQuerySharedProjectionDisposalStop<D, O, F, L: BasisOperationLane> {
    lease: WorthQuerySharedLiveProjectionLease<D, O, F, L>,
    error: WorthQueryRuntimeError,
    counters: crate::runtime::WorthQuerySharedLeaseReleaseCounters,
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySharedProjectionDisposalStop<D, O, F, L> {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub const fn counters(&self) -> crate::runtime::WorthQuerySharedLeaseReleaseCounters {
        self.counters
    }

    pub fn into_lease(self) -> WorthQuerySharedLiveProjectionLease<D, O, F, L> {
        self.lease
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQuerySharedLiveProjectionLease<D, O, F, L>,
        WorthQueryRuntimeError,
        crate::runtime::WorthQuerySharedLeaseReleaseCounters,
    ) {
        (self.lease, self.error, self.counters)
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySharedLiveProjectionLease<D, O, F, L> {
    pub fn dispose(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQuerySharedProjectionDisposalOutcome<D, O, F, L> {
        worth_execution::run_with_execution_stack(|| self.dispose_on_execution_stack(workspace))
    }

    #[inline(never)]
    fn dispose_on_execution_stack(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQuerySharedProjectionDisposalOutcome<D, O, F, L> {
        let (source, proof, capability, token, admission_counters) = self.into_parts();
        match workspace.release_shared_projection_lease(&capability, token) {
            Ok(release) => WorthQuerySharedProjectionDisposalOutcome::Disposed(
                WorthQueryDisposedSharedProjection { source, release },
            ),
            Err(stopped) => WorthQuerySharedProjectionDisposalOutcome::Stopped(
                WorthQuerySharedProjectionDisposalStop {
                    lease: WorthQuerySharedLiveProjectionLease::from_parts(
                        source,
                        proof,
                        capability,
                        stopped.token,
                        admission_counters,
                    ),
                    error: stopped.error,
                    counters: stopped.counters,
                },
            ),
        }
    }
}
