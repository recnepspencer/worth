use super::{
    progress::WorthQueryBranchSetAdoptionProgress, WorthQueryOrderedProgramAdoptionCoverage,
};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};
use crate::domain_computation::primary_graph::{
    WorthQueryBranchAdoptionPreparationDenial, WorthQueryBranchAdoptionPublicationOutcome,
    WorthQueryPreparedBranchAdoption, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::collections::VecDeque;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;
mod admission;
mod recovery;
mod resume;
pub use recovery::{
    WorthQueryBranchSetAdoptionRecovery, WorthQueryBranchSetAdoptionRecoveryFailure,
    WorthQueryBranchSetAdoptionRecoveryOutcome, WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
};
pub use resume::{
    WorthQueryBranchSetAdoptionResumeDenial, WorthQueryBranchSetAdoptionResumeFailure,
    WorthQueryStoppedBranchSetAdoption,
};
/// Why a branch-set adoption could not be prepared. Nothing was published.
#[derive(Debug)]
pub enum WorthQueryBranchSetAdoptionPreparationDenial {
    DurableStateAdmission(crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial),
    /// The preparation call could not enter its execution request.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// Memory to hold the prepared branches could not be allocated.
    RetentionAllocationRejected,
    /// A covered branch could not be selected.
    ProductSelection {
        branch: WorthQueryProductBranch,
        denial: WorthQueryProductBranchAdmissionDenial,
    },
    /// A covered branch's adoption could not be prepared.
    Adoption {
        branch: WorthQueryProductBranch,
        denial: WorthQueryBranchAdoptionPreparationDenial,
    },
    /// The total selection work could not be counted without overflow.
    WorkAccountingOverflow,
}
impl WorthQueryBranchSetAdoptionPreparationDenial {
    pub const fn branch(&self) -> Option<WorthQueryProductBranch> {
        match self {
            Self::ProductSelection { branch, .. } | Self::Adoption { branch, .. } => Some(*branch),
            Self::DurableStateAdmission(_)
            | Self::ExecutionDenied(_)
            | Self::RetentionAllocationRejected
            | Self::WorkAccountingOverflow => None,
        }
    }
}

pub(super) struct PendingBranchAdoption {
    pub(super) branch: WorthQueryProductBranch,
    pub(super) adoption: WorthQueryPreparedBranchAdoption,
}

#[derive(Clone, Copy)]
pub(super) enum BranchSetAdoptionResolution {
    NoEffect {
        branch: WorthQueryProductBranch,
        source: ApplicationProgramRevision,
        attempt: Option<worth_foundational::facade::DurableCompositePublicationAttemptIdentity>,
    },
    ProductUnpublished(WorthQueryProductBranch),
}

impl BranchSetAdoptionResolution {
    pub(super) const fn branch(self) -> WorthQueryProductBranch {
        match self {
            Self::NoEffect { branch, .. } | Self::ProductUnpublished(branch) => branch,
        }
    }
}

/// Why a branch-set adoption would not advance to the next branch. Nothing was
/// published.
#[derive(Debug)]
pub enum WorthQueryBranchSetAdoptionAdvanceDenial {
    ObligationPublication {
        branch: WorthQueryProductBranch,
        cause: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    },
    Execution {
        branch: WorthQueryProductBranch,
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    },
    Authorization {
        branch: WorthQueryProductBranch,
        cause: super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial,
    },
    /// The last branch's publication had no effect or left the product
    /// unpublished. Resume, recover, or cancel before advancing.
    ResolutionRequired { branch: WorthQueryProductBranch },
}

/// Why a branch-set adoption cannot be closed yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchSetAdoptionCloseDenial {
    /// Some branches are not yet published; `remaining` counts them.
    Pending { remaining: usize },
    /// The last branch's publication had no effect or left the product
    /// unpublished. Resume, recover, or cancel it first.
    ResolutionRequired { branch: WorthQueryProductBranch },
}

/// Bounded, explicitly non-atomic progression over preflighted branch moves.
#[must_use = "a branch-set adoption retains prepared publications until advanced or cancelled"]
pub struct WorthQueryPreparedBranchSetAdoption {
    pub(super) pending: VecDeque<PendingBranchAdoption>,
    pub(super) progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    pub(super) resolution_required: Option<BranchSetAdoptionResolution>,
    pub(super) target: ApplicationProgramRevision,
    pub(super) total_selection_work_units: usize,
}

impl WorthQueryPreparedBranchSetAdoption {
    pub fn pending_branches(&self) -> impl Iterator<Item = WorthQueryProductBranch> + '_ {
        self.pending.iter().map(|pending| pending.branch)
    }

    /// Returns the current disposition for each branch already attempted.
    /// A successful resume replaces its superseded no-effect disposition.
    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        &self.progress
    }

    pub const fn total_selection_work_units(&self) -> usize {
        self.total_selection_work_units
    }

    pub fn advance<Schema: ApplicationSchema>(
        &mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Option<&WorthQueryBranchSetAdoptionProgress>,
        WorthQueryBranchSetAdoptionAdvanceDenial,
    > {
        if let Some(resolution) = self.resolution_required {
            return Err(
                WorthQueryBranchSetAdoptionAdvanceDenial::ResolutionRequired {
                    branch: resolution.branch(),
                },
            );
        }
        let Some(pending) = self.pending.pop_front() else {
            return Ok(None);
        };
        let source = *pending.adoption.source();
        let planned = pending.adoption.publication.planned_successor();
        let reserved = planned.attempt_identity();
        let attempt = reserved.owner_identity().durable().map(|owner| {
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                reserved.ordinal(),
            )
        });
        let outcome = pending.adoption.publish(application, principal, request);
        let (outcome, discharge_denial) = match outcome {
            WorthQueryBranchAdoptionPublicationOutcome::ObligationDischargePending {
                cause,
                outcome,
            } => (*outcome, Some(cause)),
            outcome => (outcome, None),
        };
        let progress = match outcome {
            WorthQueryBranchAdoptionPublicationOutcome::ObligationDischargePending { .. } => {
                unreachable!("one native disposition is wrapped once")
            }
            WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied { cause, prepared } => {
                self.pending.push_front(PendingBranchAdoption {
                    branch: pending.branch,
                    adoption: prepared,
                });
                return Err(WorthQueryBranchSetAdoptionAdvanceDenial::Execution {
                    branch: pending.branch,
                    cause,
                });
            }
            WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
                self.pending.push_front(PendingBranchAdoption {
                    branch: pending.branch,
                    adoption: prepared,
                });
                return Err(WorthQueryBranchSetAdoptionAdvanceDenial::Authorization {
                    branch: pending.branch,
                    cause,
                });
            }
            WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(cause) => {
                WorthQueryBranchSetAdoptionProgress::ExecutionDenied {
                    branch: pending.branch,
                    cause,
                }
            }
            WorthQueryBranchAdoptionPublicationOutcome::Performed(adoption) => {
                WorthQueryBranchSetAdoptionProgress::Performed {
                    branch: pending.branch,
                    adoption,
                }
            }
            WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
                WorthQueryBranchSetAdoptionProgress::NoEffect {
                    branch: pending.branch,
                    no_effect,
                }
            }
            WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(adoption) => {
                WorthQueryBranchSetAdoptionProgress::ProductUnpublished {
                    branch: pending.branch,
                    adoption,
                }
            }
        };
        self.resolution_required = match &progress {
            WorthQueryBranchSetAdoptionProgress::Performed { .. }
            | WorthQueryBranchSetAdoptionProgress::RecordedPerformed { .. } => None,
            WorthQueryBranchSetAdoptionProgress::RecordedNoEffect { .. } => {
                unreachable!("only cold stopped custody carries recorded stops")
            }
            WorthQueryBranchSetAdoptionProgress::ExecutionDenied { branch, .. }
            | WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } => {
                Some(BranchSetAdoptionResolution::NoEffect {
                    branch: *branch,
                    source,
                    attempt,
                })
            }
            WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, .. } => {
                Some(BranchSetAdoptionResolution::ProductUnpublished(*branch))
            }
        };
        self.progress.push(progress);
        if let Some(cause) = discharge_denial {
            return Err(
                WorthQueryBranchSetAdoptionAdvanceDenial::ObligationPublication {
                    branch: pending.branch,
                    cause,
                },
            );
        }
        Ok(self.progress.last())
    }

    pub fn close_denial(&self) -> Option<WorthQueryBranchSetAdoptionCloseDenial> {
        if let Some(resolution) = self.resolution_required {
            return Some(WorthQueryBranchSetAdoptionCloseDenial::ResolutionRequired {
                branch: resolution.branch(),
            });
        }
        (!self.pending.is_empty()).then_some(WorthQueryBranchSetAdoptionCloseDenial::Pending {
            remaining: self.pending.len(),
        })
    }

    pub fn close(self) -> Result<WorthQueryClosedBranchSetAdoption, Self> {
        if self.close_denial().is_some() {
            return Err(self);
        }
        Ok(WorthQueryClosedBranchSetAdoption {
            progress: self.progress,
            total_selection_work_units: self.total_selection_work_units,
        })
    }

    /// Cancels only branches whose owner effects have not started. Completed
    /// branches remain completed; a no-effect stop is resolved by cancellation,
    /// while unpublished owner custody must be recovered first.
    pub fn cancel(self) -> Result<WorthQueryBranchSetAdoptionCancellation, Self> {
        if matches!(
            self.resolution_required,
            Some(BranchSetAdoptionResolution::ProductUnpublished(_))
        ) {
            return Err(self);
        }
        Ok(WorthQueryBranchSetAdoptionCancellation {
            cancelled_branch_count: self.pending.len()
                + usize::from(self.resolution_required.is_some()),
            progress: self.progress,
            total_selection_work_units: self.total_selection_work_units,
        })
    }
}

/// A branch-set adoption in which every covered branch is published. Returned
/// by `close`.
pub struct WorthQueryClosedBranchSetAdoption {
    progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    total_selection_work_units: usize,
}

impl WorthQueryClosedBranchSetAdoption {
    /// Returns one terminal performed disposition per covered branch.
    /// Superseded no-effect attempts remain reflected in cumulative work only.
    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        &self.progress
    }

    pub const fn total_selection_work_units(&self) -> usize {
        self.total_selection_work_units
    }
}

/// A cancelled branch-set adoption: how many branches it did not publish, and
/// the progress made before cancelling. Published branches stay published.
pub struct WorthQueryBranchSetAdoptionCancellation {
    pub(super) cancelled_branch_count: usize,
    pub(super) progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    pub(super) total_selection_work_units: usize,
}

impl WorthQueryBranchSetAdoptionCancellation {
    pub const fn cancelled_branch_count(&self) -> usize {
        self.cancelled_branch_count
    }

    /// Returns completed dispositions plus any active no-effect stop at cancellation.
    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        &self.progress
    }

    pub const fn total_selection_work_units(&self) -> usize {
        self.total_selection_work_units
    }
}

mod preparation;
