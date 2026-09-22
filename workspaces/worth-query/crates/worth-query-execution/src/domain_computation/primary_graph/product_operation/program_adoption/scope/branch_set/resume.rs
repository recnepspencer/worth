use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    BranchSetAdoptionResolution, WorthQueryBranchSetAdoptionCancellation,
    WorthQueryBranchSetAdoptionPreparationDenial, WorthQueryPreparedBranchSetAdoption,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    WorthQueryBranchSetAdoptionProgress, WorthQueryOrderedProgramAdoptionCoverage,
    WorthQueryPrimaryGraphApplicationRuntime,
};

/// A stopped non-atomic adoption whose old prepared suffix has been released.
/// Resumption requires fresh owner-issued coverage and fresh preparation for
/// every branch that did not perform.
#[must_use = "a stopped branch-set adoption must be resumed or cancelled"]
pub struct WorthQueryStoppedBranchSetAdoption {
    remaining: Box<[WorthQueryProductBranch]>,
    progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    target: ApplicationProgramRevision,
    total_selection_work_units: usize,
}

impl WorthQueryStoppedBranchSetAdoption {
    pub fn remaining_branches(&self) -> &[WorthQueryProductBranch] {
        &self.remaining
    }

    /// Returns the performed prefix followed by the active no-effect disposition.
    /// A successful fresh resume supersedes that final no-effect entry.
    pub fn progress(&self) -> &[WorthQueryBranchSetAdoptionProgress] {
        &self.progress
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub const fn total_selection_work_units(&self) -> usize {
        self.total_selection_work_units
    }

    /// Cancels every branch retained by the stopped adoption.
    ///
    /// This is infallible because stopping releases every prepared publication
    /// and is available only for a no-effect outcome, never unpublished custody.
    pub fn cancel(self) -> WorthQueryBranchSetAdoptionCancellation {
        WorthQueryBranchSetAdoptionCancellation {
            cancelled_branch_count: self.remaining.len(),
            progress: self.progress,
            total_selection_work_units: self.total_selection_work_units,
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryBranchSetAdoptionResumeDenial {
    CoverageMismatch,
    RetentionAllocationRejected,
    Preparation(WorthQueryBranchSetAdoptionPreparationDenial),
    WorkAccountingOverflow,
}

pub struct WorthQueryBranchSetAdoptionResumeFailure {
    denial: WorthQueryBranchSetAdoptionResumeDenial,
    adoption: WorthQueryStoppedBranchSetAdoption,
}

impl WorthQueryBranchSetAdoptionResumeFailure {
    pub const fn denial(&self) -> &WorthQueryBranchSetAdoptionResumeDenial {
        &self.denial
    }

    pub fn into_adoption(self) -> WorthQueryStoppedBranchSetAdoption {
        self.adoption
    }
}

impl WorthQueryPreparedBranchSetAdoption {
    /// Releases the obsolete prepared suffix while retaining exact progress,
    /// target meaning, and the ordered set that must be freshly admitted.
    pub fn begin_resume(self) -> Result<WorthQueryStoppedBranchSetAdoption, Self> {
        let Some(BranchSetAdoptionResolution::NoEffect(blocked)) = self.resolution_required else {
            return Err(self);
        };
        let mut remaining = Vec::new();
        if remaining.try_reserve_exact(self.pending.len() + 1).is_err() {
            return Err(self);
        }
        remaining.push(blocked);
        remaining.extend(self.pending.iter().map(|pending| pending.branch));
        Ok(WorthQueryStoppedBranchSetAdoption {
            remaining: remaining.into_boxed_slice(),
            progress: self.progress,
            target: self.target,
            total_selection_work_units: self.total_selection_work_units,
        })
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn resume_branch_set_adoption(
        &self,
        mut adoption: WorthQueryStoppedBranchSetAdoption,
        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        maximum_selection_work_per_branch: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionResumeFailure> {
        if coverage.branches() != adoption.remaining.as_ref() {
            return Err(resume_failure(
                WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch,
                adoption,
            ));
        }
        let (pending, resumed_work) = match self.prepare_branch_set_targets(
            coverage.branches(),
            &adoption.target,
            maximum_selection_work_per_branch,
            request,
        ) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return Err(resume_failure(
                    WorthQueryBranchSetAdoptionResumeDenial::Preparation(denial),
                    adoption,
                ))
            }
        };
        if adoption.progress.try_reserve_exact(pending.len()).is_err() {
            return Err(resume_failure(
                WorthQueryBranchSetAdoptionResumeDenial::RetentionAllocationRejected,
                adoption,
            ));
        }
        let Some(total_selection_work_units) = adoption
            .total_selection_work_units
            .checked_add(resumed_work)
        else {
            return Err(resume_failure(
                WorthQueryBranchSetAdoptionResumeDenial::WorkAccountingOverflow,
                adoption,
            ));
        };
        let superseded = adoption
            .progress
            .pop()
            .expect("a stopped branch-set adoption retains its active no-effect disposition");
        assert!(
            matches!(
                &superseded,
                WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. }
                    if Some(*branch) == adoption.remaining.first().copied()
            ),
            "a stopped branch-set adoption retains the no-effect disposition for its first remaining branch"
        );
        Ok(WorthQueryPreparedBranchSetAdoption {
            pending,
            progress: adoption.progress,
            resolution_required: None,
            target: adoption.target,
            total_selection_work_units,
        })
    }
}

fn resume_failure(
    denial: WorthQueryBranchSetAdoptionResumeDenial,
    adoption: WorthQueryStoppedBranchSetAdoption,
) -> WorthQueryBranchSetAdoptionResumeFailure {
    WorthQueryBranchSetAdoptionResumeFailure { denial, adoption }
}
