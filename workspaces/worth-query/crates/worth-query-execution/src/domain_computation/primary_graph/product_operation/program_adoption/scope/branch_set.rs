use std::collections::VecDeque;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    progress::WorthQueryBranchSetAdoptionProgress, WorthQueryOrderedProgramAdoptionCoverage,
};
use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial};
use crate::domain_computation::primary_graph::{
    WorthQueryBranchAdoptionPreparationDenial, WorthQueryBranchAdoptionPublicationOutcome,
    WorthQueryPreparedBranchAdoption, WorthQueryPrimaryGraphApplicationRuntime,
};

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

#[derive(Debug)]
pub enum WorthQueryBranchSetAdoptionPreparationDenial {
    RetentionAllocationRejected,
    ProductSelection {
        branch: WorthQueryProductBranch,
        denial: WorthQueryProductBranchAdmissionDenial,
    },
    Adoption {
        branch: WorthQueryProductBranch,
        denial: WorthQueryBranchAdoptionPreparationDenial,
    },
    WorkAccountingOverflow,
}

impl WorthQueryBranchSetAdoptionPreparationDenial {
    pub const fn branch(&self) -> Option<WorthQueryProductBranch> {
        match self {
            Self::ProductSelection { branch, .. } | Self::Adoption { branch, .. } => Some(*branch),
            Self::RetentionAllocationRejected | Self::WorkAccountingOverflow => None,
        }
    }
}

pub(super) struct PendingBranchAdoption {
    pub(super) branch: WorthQueryProductBranch,
    pub(super) adoption: WorthQueryPreparedBranchAdoption,
}

#[derive(Clone, Copy)]
pub(super) enum BranchSetAdoptionResolution {
    NoEffect(WorthQueryProductBranch),
    ProductUnpublished(WorthQueryProductBranch),
}

impl BranchSetAdoptionResolution {
    pub(super) const fn branch(self) -> WorthQueryProductBranch {
        match self {
            Self::NoEffect(branch) | Self::ProductUnpublished(branch) => branch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchSetAdoptionAdvanceDenial {
    ResolutionRequired { branch: WorthQueryProductBranch },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchSetAdoptionCloseDenial {
    Pending { remaining: usize },
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

    pub fn advance(
        &mut self,
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
        let progress = match pending.adoption.publish() {
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
            WorthQueryBranchSetAdoptionProgress::Performed { .. } => None,
            WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } => {
                Some(BranchSetAdoptionResolution::NoEffect(*branch))
            }
            WorthQueryBranchSetAdoptionProgress::ProductUnpublished { branch, .. } => {
                Some(BranchSetAdoptionResolution::ProductUnpublished(*branch))
            }
        };
        self.progress.push(progress);
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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn prepare_branch_set_adoption(
        &self,
        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        target: &ApplicationProgramRevision,
        maximum_selection_work_per_branch: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionPreparationDenial>
    {
        let (pending, total_selection_work_units) = self.prepare_branch_set_targets(
            &coverage.branches,
            target,
            maximum_selection_work_per_branch,
            request,
        )?;
        let mut progress = Vec::new();
        progress
            .try_reserve_exact(coverage.branches.len())
            .map_err(|_| {
                WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
            })?;
        Ok(WorthQueryPreparedBranchSetAdoption {
            pending,
            progress,
            resolution_required: None,
            target: target.clone(),
            total_selection_work_units,
        })
    }

    pub(super) fn prepare_branch_set_targets(
        &self,
        branches: &[WorthQueryProductBranch],
        target: &ApplicationProgramRevision,
        maximum_selection_work_per_branch: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<
        (VecDeque<PendingBranchAdoption>, usize),
        WorthQueryBranchSetAdoptionPreparationDenial,
    > {
        let mut pending = VecDeque::new();
        pending.try_reserve(branches.len()).map_err(|_| {
            WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
        })?;
        let mut total_selection_work_units = 0usize;
        for &branch in branches {
            let selected = self.on_branch(branch).select().map_err(|denial| {
                WorthQueryBranchSetAdoptionPreparationDenial::ProductSelection { branch, denial }
            })?;
            let requirements = selected
                .branch_adoption_requirements(target)
                .map_err(
                    |denial| WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                        branch,
                        denial,
                    },
                )?;
            let adoption = selected
                .prepare_branch_adoption(
                    target,
                    &requirements,
                    maximum_selection_work_per_branch,
                    request,
                )
                .map_err(
                    |denial| WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                        branch,
                        denial,
                    },
                )?;
            total_selection_work_units = total_selection_work_units
                .checked_add(adoption.selection_work_units())
                .ok_or(WorthQueryBranchSetAdoptionPreparationDenial::WorkAccountingOverflow)?;
            pending.push_back(PendingBranchAdoption { branch, adoption });
        }
        Ok((pending, total_selection_work_units))
    }
}
