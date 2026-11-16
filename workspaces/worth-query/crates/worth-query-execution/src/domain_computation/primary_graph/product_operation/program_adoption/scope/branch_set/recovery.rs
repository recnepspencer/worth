use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::NoEffectCompositePublication;

use super::{
    BranchSetAdoptionResolution, WorthQueryBranchSetAdoptionCancellation,
    WorthQueryPreparedBranchSetAdoption,
};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanupFailure, WorthQueryProductBranchOwnerCleanupReceipt,
};
use crate::domain_computation::primary_graph::product_operation::program_adoption::custody::WorthQueryBranchAdoptionRecoveryReleaseFailure;
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

    /// Returns completed dispositions before the branch under recovery.
    /// That branch's disposition remains in this recovery's separate custody.
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

/// What continuing a branch-set adoption recovery did. Returned by
/// `recover_branch_set_adoption`.
pub enum WorthQueryBranchSetAdoptionRecoveryOutcome {
    ObligationDischargePending {
        cause: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
        outcome: Box<WorthQueryBranchSetAdoptionRecoveryOutcome>,
    },
    /// The blocked branch is published, and the branch-set adoption is handed
    /// back to advance. `cleanup` reports releasing the recovery's owner
    /// custody.
    Performed {
        adoption: WorthQueryPreparedBranchSetAdoption,
        cleanup: Result<
            WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
    /// The publication had no effect: nothing was published. The recovery is
    /// handed back to continue again.
    NoEffect {
        no_effect: NoEffectCompositePublication,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    /// The publication again moved some owners but not the product head. The
    /// new recovery is handed back; `prior_cleanup` reports releasing the
    /// earlier recovery's custody.
    ProductUnpublished {
        recovery: WorthQueryBranchSetAdoptionRecovery,
        prior_cleanup: Result<
            WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
}

/// Why continuing a branch-set adoption recovery stopped. Nothing was
/// published; the recovery can be taken back with `into_recovery`.
pub enum WorthQueryBranchSetAdoptionRecoveryFailure {
    /// The blocked branch could not be selected.
    ProductSelection {
        denial: WorthQueryProductBranchAdmissionDenial,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    /// The branch adoption recovery stopped; `failure` says why.
    Recovery {
        branch: WorthQueryProductBranch,
        adoption: WorthQueryPreparedBranchSetAdoption,
        failure: WorthQueryBranchAdoptionRecoveryFailure,
    },
}

/// Why `release_branch_set_adoption_recovery` did not return a cleanup receipt.
pub enum WorthQueryBranchSetAdoptionRecoveryReleaseFailure {
    ObligationDischargePending {
        cause: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
        cancellation: WorthQueryBranchSetAdoptionCancellation,
        cleanup: Result<
            WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductBranchOwnerCleanupFailure,
        >,
    },
    /// The recovery record was not released; the recovery is handed back
    /// unchanged to retry.
    Recovery {
        denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
        recovery: WorthQueryBranchSetAdoptionRecovery,
    },
    /// The recovery record was released and the remaining branches were
    /// cancelled, but owner cleanup did not finish. The cleanup is carried for
    /// retry.
    OwnerCleanup {
        cancellation: WorthQueryBranchSetAdoptionCancellation,
        failure: WorthQueryProductBranchOwnerCleanupFailure,
    },
}

impl WorthQueryBranchSetAdoptionRecoveryReleaseFailure {
    pub fn into_recovery(self) -> Option<WorthQueryBranchSetAdoptionRecovery> {
        match self {
            Self::Recovery { recovery, .. } => Some(recovery),
            Self::OwnerCleanup { .. } | Self::ObligationDischargePending { .. } => None,
        }
    }

    pub fn into_owner_cleanup(
        self,
    ) -> Option<(
        WorthQueryBranchSetAdoptionCancellation,
        WorthQueryProductBranchOwnerCleanupFailure,
    )> {
        match self {
            Self::Recovery { .. } | Self::ObligationDischargePending { cleanup: Ok(_), .. } => None,
            Self::ObligationDischargePending {
                cancellation,
                cleanup: Err(failure),
                ..
            } => Some((cancellation, failure)),
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
        if !matches!(
            self.resolution_required,
            Some(BranchSetAdoptionResolution::ProductUnpublished(_))
        ) {
            return Err(self);
        }
        let Some(progress) = self.progress.pop() else {
            return Err(self);
        };
        match progress {
            WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, adoption } => Ok({
                self.resolution_required = None;
                WorthQueryBranchSetAdoptionRecovery::unpublished(branch, self, adoption)
            }),
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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryBranchSetAdoptionRecoveryOutcome,
        WorthQueryBranchSetAdoptionRecoveryFailure,
    > {
        let phase_request = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => {
                return Err(
                    WorthQueryBranchSetAdoptionRecoveryFailure::ProductSelection {
                        denial: WorthQueryProductBranchAdmissionDenial::Refused(
                            AdmissionRefusal::ForeignPhase,
                        ),
                        recovery,
                    },
                )
            }
        };
        let execution = &phase_request;
        let (branch, mut adoption, recovery) = recovery.into_parts();
        let selected = match self.on_branch(branch).select_with_request(execution) {
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
        let recovered = selected
            .recover_branch_adoption_authorized_in_advancement(phase, recovery, principal, request);
        let (recovered, discharge) = match recovered {
            Ok(WorthQueryBranchAdoptionRecoveryOutcome::ObligationDischargePending {
                cause,
                outcome,
            }) => (Ok(*outcome), Some(cause)),
            result => (result, None),
        };
        let result = match recovered {
            Ok(WorthQueryBranchAdoptionRecoveryOutcome::ObligationDischargePending { .. }) => {
                unreachable!("one native disposition is wrapped once")
            }
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
        };
        result.map(|outcome| match discharge {
            Some(cause) => WorthQueryBranchSetAdoptionRecoveryOutcome::ObligationDischargePending {
                cause,
                outcome: Box::new(outcome),
            },
            None => outcome,
        })
    }
}

mod release;
