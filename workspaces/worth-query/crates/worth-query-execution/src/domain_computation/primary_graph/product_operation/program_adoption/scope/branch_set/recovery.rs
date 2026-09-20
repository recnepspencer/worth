use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::NoEffectCompositePublication;

use super::{WorthQueryBranchSetAdoptionCancellation, WorthQueryPreparedBranchSetAdoption};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanupFailure, WorthQueryProductBranchOwnerCleanupReceipt,
};
use crate::domain_computation::primary_graph::product_operation::program_adoption::custody::WorthQueryBranchAdoptionCustodyReleaseFailure;
use crate::domain_computation::primary_graph::{
    WorthQueryBranchAdoptionRecovery, WorthQueryBranchAdoptionRecoveryFailure,
    WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryUnpublishedBranchAdoption,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

use super::super::progress::WorthQueryBranchSetAdoptionProgress;

enum WorthQueryBranchSetRecoveryCustody {
    Unpublished(WorthQueryUnpublishedBranchAdoption),
    Recovery(WorthQueryBranchAdoptionRecovery),
}

/// Exact unpublished custody together with the performed prefix and untouched suffix.
#[must_use = "branch-set recovery retains exact owner custody and pending branches"]
pub struct WorthQueryBranchSetAdoptionRecovery {
    branch: WorthQueryProductBranch,
    adoption: WorthQueryPreparedBranchSetAdoption,
    custody: WorthQueryBranchSetRecoveryCustody,
}

impl WorthQueryBranchSetAdoptionRecovery {
    fn unpublished(
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPreparedBranchSetAdoption,
        unpublished: WorthQueryUnpublishedBranchAdoption,
    ) -> Self {
        Self {
            branch,
            adoption,
            custody: WorthQueryBranchSetRecoveryCustody::Unpublished(unpublished),
        }
    }

    fn retained(
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPreparedBranchSetAdoption,
        recovery: WorthQueryBranchAdoptionRecovery,
    ) -> Self {
        Self {
            branch,
            adoption,
            custody: WorthQueryBranchSetRecoveryCustody::Recovery(recovery),
        }
    }

    pub const fn branch(&self) -> WorthQueryProductBranch {
        self.branch
    }

    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        self.adoption.progress()
    }

    pub fn pending_branches(&self) -> impl Iterator<Item = WorthQueryProductBranch> + '_ {
        self.adoption.pending_branches()
    }

    pub const fn unpublished_custody(&self) -> Option<&WorthQueryUnpublishedBranchAdoption> {
        match &self.custody {
            WorthQueryBranchSetRecoveryCustody::Unpublished(unpublished) => Some(unpublished),
            WorthQueryBranchSetRecoveryCustody::Recovery(_) => None,
        }
    }

    fn into_parts(
        self,
    ) -> (
        WorthQueryProductBranch,
        WorthQueryPreparedBranchSetAdoption,
        WorthQueryBranchAdoptionRecovery,
    ) {
        let recovery = match self.custody {
            WorthQueryBranchSetRecoveryCustody::Unpublished(unpublished) => {
                unpublished.into_recovery()
            }
            WorthQueryBranchSetRecoveryCustody::Recovery(recovery) => recovery,
        };
        (self.branch, self.adoption, recovery)
    }
}

pub enum WorthQueryBranchSetAdoptionRecoveryOutcome {
    Performed {
        adoption: WorthQueryPreparedBranchSetAdoption,
        cleanup: Result<
            WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
    NoEffect {
        no_effect: NoEffectCompositePublication,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    ProductUnpublished {
        recovery: WorthQueryBranchSetAdoptionRecovery,
        prior_cleanup: Result<
            WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
}

pub enum WorthQueryBranchSetAdoptionRecoveryFailure {
    ProductSelection {
        denial: WorthQueryProductBranchAdmissionDenial,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    Recovery {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPreparedBranchSetAdoption,
        failure: WorthQueryBranchAdoptionRecoveryFailure,
    },
}

pub enum WorthQueryBranchSetAdoptionRecoveryReleaseFailure {
    Recovery {
        denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    OwnerCleanup {
        cancellation: WorthQueryBranchSetAdoptionCancellation,
        failure: WorthQueryProductBranchOwnerCleanupFailure,
    },
}

impl WorthQueryBranchSetAdoptionRecoveryReleaseFailure {
    pub fn into_recovery(self) -> Option<WorthQueryBranchSetAdoptionRecovery> {
        match self {
            Self::Recovery { recovery, .. } => Some(recovery),
            Self::OwnerCleanup { .. } => None,
        }
    }

    pub fn into_owner_cleanup(
        self,
    ) -> Option<(
        WorthQueryBranchSetAdoptionCancellation,
        WorthQueryProductBranchOwnerCleanupFailure,
    )> {
        match self {
            Self::Recovery { .. } => None,
            Self::OwnerCleanup {
                cancellation,
                failure,
            } => Some((cancellation, failure)),
        }
    }
}

impl WorthQueryBranchSetAdoptionRecoveryFailure {
    pub const fn branch(&self) -> WorthQueryProductBranch {
        match self {
            Self::ProductSelection { recovery, .. } => recovery.branch(),
            Self::Recovery { branch, .. } => *branch,
        }
    }

    pub fn into_recovery(self) -> WorthQueryBranchSetAdoptionRecovery {
        match self {
            Self::ProductSelection { recovery, .. } => recovery,
            Self::Recovery {
                branch,
                adoption,
                failure,
            } => WorthQueryBranchSetAdoptionRecovery::retained(
                branch,
                adoption,
                failure.into_recovery(),
            ),
        }
    }
}

impl WorthQueryPreparedBranchSetAdoption {
    /// Transfers the blocking unpublished terminal into a recovery object that
    /// keeps the performed prefix and untouched suffix inseparable from it.
    pub fn begin_recovery(mut self) -> Result<WorthQueryBranchSetAdoptionRecovery, Self> {
        let Some(progress) = self.progress.pop() else {
            return Err(self);
        };
        match progress {
            WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, adoption } => Ok(
                WorthQueryBranchSetAdoptionRecovery::unpublished(branch, self, adoption),
            ),
            progress => {
                self.progress.push(progress);
                Err(self)
            }
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn recover_branch_set_adoption(
        &self,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryBranchSetAdoptionRecoveryOutcome,
        WorthQueryBranchSetAdoptionRecoveryFailure,
    > {
        let (branch, mut adoption, recovery) = recovery.into_parts();
        let selected = match self.on_branch(branch).select() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(
                    WorthQueryBranchSetAdoptionRecoveryFailure::ProductSelection {
                        denial,
                        recovery: WorthQueryBranchSetAdoptionRecovery::retained(
                            branch, adoption, recovery,
                        ),
                    },
                )
            }
        };
        match selected.recover_branch_adoption(recovery, request) {
            Ok(WorthQueryBranchAdoptionRecoveryOutcome::Performed {
                adoption: performed,
                cleanup,
            }) => {
                adoption
                    .progress
                    .push(WorthQueryBranchSetAdoptionProgress::Performed {
                        branch,
                        adoption: performed,
                    });
                Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::Performed { adoption, cleanup })
            }
            Ok(WorthQueryBranchAdoptionRecoveryOutcome::NoEffect {
                no_effect,
                recovery,
            }) => Ok(WorthQueryBranchSetAdoptionRecoveryOutcome::NoEffect {
                no_effect,
                recovery: WorthQueryBranchSetAdoptionRecovery::retained(branch, adoption, recovery),
            }),
            Ok(WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished {
                next,
                prior_cleanup,
            }) => Ok(
                WorthQueryBranchSetAdoptionRecoveryOutcome::ProductUnpublished {
                    recovery: WorthQueryBranchSetAdoptionRecovery::unpublished(
                        branch, adoption, next,
                    ),
                    prior_cleanup,
                },
            ),
            Err(failure) => Err(WorthQueryBranchSetAdoptionRecoveryFailure::Recovery {
                branch,
                adoption,
                failure,
            }),
        }
    }

    pub fn release_branch_set_adoption_recovery(
        &self,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        (
            WorthQueryBranchSetAdoptionCancellation,
            WorthQueryProductBranchOwnerCleanupReceipt,
        ),
        WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
    > {
        let (branch, adoption, recovery) = recovery.into_parts();
        match self.release_branch_adoption_custody(recovery, minimum_age_ticks) {
            Ok(receipt) => Ok((cancel_unstarted(adoption), receipt)),
            Err(WorthQueryBranchAdoptionCustodyReleaseFailure::Recovery { denial, recovery }) => {
                Err(
                    WorthQueryBranchSetAdoptionRecoveryReleaseFailure::Recovery {
                        denial,
                        recovery: WorthQueryBranchSetAdoptionRecovery::retained(
                            branch, adoption, recovery,
                        ),
                    },
                )
            }
            Err(WorthQueryBranchAdoptionCustodyReleaseFailure::OwnerCleanup(failure)) => Err(
                WorthQueryBranchSetAdoptionRecoveryReleaseFailure::OwnerCleanup {
                    cancellation: cancel_unstarted(adoption),
                    failure,
                },
            ),
        }
    }
}

fn cancel_unstarted(
    adoption: WorthQueryPreparedBranchSetAdoption,
) -> WorthQueryBranchSetAdoptionCancellation {
    WorthQueryBranchSetAdoptionCancellation {
        cancelled_pending_branch_count: adoption.pending.len(),
        progress: adoption.progress,
        total_selection_work_units: adoption.total_selection_work_units,
    }
}
