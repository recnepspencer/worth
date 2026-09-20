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
pub use recovery::{
    WorthQueryBranchSetAdoptionRecovery, WorthQueryBranchSetAdoptionRecoveryFailure,
    WorthQueryBranchSetAdoptionRecoveryOutcome, WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
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

struct PendingBranchAdoption {
    branch: WorthQueryProductBranch,
    adoption: WorthQueryPreparedBranchAdoption,
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
    pending: VecDeque<PendingBranchAdoption>,
    progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    total_selection_work_units: usize,
}

impl WorthQueryPreparedBranchSetAdoption {
    pub fn pending_branches(&self) -> impl Iterator<Item = WorthQueryProductBranch> + '_ {
        self.pending.iter().map(|pending| pending.branch)
    }

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
        if let Some(blocked) = self.progress.last().filter(|entry| entry.blocks_advance()) {
            return Err(
                WorthQueryBranchSetAdoptionAdvanceDenial::ResolutionRequired {
                    branch: blocked.branch(),
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
        self.progress.push(progress);
        Ok(self.progress.last())
    }

    pub fn close_denial(&self) -> Option<WorthQueryBranchSetAdoptionCloseDenial> {
        if let Some(blocked) = self.progress.last().filter(|entry| entry.blocks_advance()) {
            return Some(WorthQueryBranchSetAdoptionCloseDenial::ResolutionRequired {
                branch: blocked.branch(),
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
        if self
            .progress
            .last()
            .is_some_and(|entry| entry.requires_recovery())
        {
            return Err(self);
        }
        Ok(WorthQueryBranchSetAdoptionCancellation {
            cancelled_pending_branch_count: self.pending.len(),
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
    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        &self.progress
    }

    pub const fn total_selection_work_units(&self) -> usize {
        self.total_selection_work_units
    }
}

pub struct WorthQueryBranchSetAdoptionCancellation {
    cancelled_pending_branch_count: usize,
    progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    total_selection_work_units: usize,
}

impl WorthQueryBranchSetAdoptionCancellation {
    pub const fn cancelled_pending_branch_count(&self) -> usize {
        self.cancelled_pending_branch_count
    }

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
        let mut pending = VecDeque::new();
        pending.try_reserve(coverage.branches.len()).map_err(|_| {
            WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
        })?;
        let mut progress = Vec::new();
        progress
            .try_reserve_exact(coverage.branches.len())
            .map_err(|_| {
                WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
            })?;
        let mut total_selection_work_units = 0usize;
        for branch in coverage.branches {
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
        Ok(WorthQueryPreparedBranchSetAdoption {
            pending,
            progress,
            total_selection_work_units,
        })
    }
}
