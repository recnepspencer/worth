use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryBranchSetAdoptionCancellation, WorthQueryBranchSetAdoptionPreparationDenial,
    WorthQueryPreparedBranchSetAdoption,
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
    pub(super) owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub(super) durable: Option<worth_foundational::facade::DurableCompositePublicationAttemptIdentity>,
    pub(super) _support_custody: Vec<crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody>,
    pub(super) remaining: Box<[WorthQueryProductBranch]>,
    pub(super) progress: Vec<WorthQueryBranchSetAdoptionProgress>,
    pub(super) target: ApplicationProgramRevision,
    pub(super) total_selection_work_units: usize,
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
}

/// Why a stopped branch-set adoption could not be resumed. Nothing was
/// published.
#[derive(Debug)]
pub enum WorthQueryBranchSetAdoptionResumeDenial {
    NotStopped,
    Inventory(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::ProductBranchInventoryDenial,
        >,
    ),
    ForeignRuntime,
    Obligation(crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial),
    /// The ordered coverage is not exactly the remaining branches, in order.
    CoverageMismatch,
    /// Memory to hold the prepared branches could not be allocated.
    RetentionAllocationRejected,
    /// Preparing a remaining branch was refused.
    Preparation(WorthQueryBranchSetAdoptionPreparationDenial),
    /// The total selection work could not be counted without overflow.
    WorkAccountingOverflow,
}

/// A refused resume, with the stopped branch-set adoption handed back
/// unchanged.
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

mod durable;
mod readmit;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn resume_branch_set_adoption(
        &self,

        adoption: WorthQueryStoppedBranchSetAdoption,
        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        maximum_selection_work_per_branch: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionResumeFailure> {
        let mut retained = Some(adoption);
        let result = self.with_application_advancement(request, |phase| {
        let phase = &phase;
            let phase_request = match phase.execution_request_for(&self.runtime.product_runtime) {
                Ok(request) => request,
                Err(cause) => return Err(resume_failure(WorthQueryBranchSetAdoptionResumeDenial::Preparation(WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause.into())), retained.take().expect("one admitted resume"))),
            };
            let execution = &phase_request;
            let mut adoption = retained.take().expect("one admitted resume");

        if adoption.owner != self.runtime.product_runtime.owner.owner_identity() {
            return Err(resume_failure(WorthQueryBranchSetAdoptionResumeDenial::ForeignRuntime,adoption));
        }
        if coverage.branches() != adoption.remaining.as_ref() {
            return Err(resume_failure(
                WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch,
                adoption,
            ));
        }
        let (pending, resumed_work) = match self.prepare_branch_set_targets(execution,
            phase,
            coverage.branches(),
            &adoption.target,
            maximum_selection_work_per_branch,
            principal,
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
        if let Some(attempt) = adoption.durable {
            if let Err(cause) = self.installed_program_support().expect("retained support").lifecycle().release_stopped_adoption(execution,attempt) {
                return Err(resume_failure(WorthQueryBranchSetAdoptionResumeDenial::Obligation(cause),adoption));
            }
        }
        let superseded = adoption
            .progress
            .pop()
            .expect("a stopped branch-set adoption retains its active no-effect disposition");
        assert!(
            matches!(
                &superseded,
                WorthQueryBranchSetAdoptionProgress::NoEffect { branch, .. } | WorthQueryBranchSetAdoptionProgress::RecordedNoEffect { branch, .. } | WorthQueryBranchSetAdoptionProgress::ProductUnpublished {branch,..} | WorthQueryBranchSetAdoptionProgress::ExecutionDenied { branch, .. }
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

        });
        match result {
            Ok(result) => result,
            Err(cause) => Err(resume_failure(
                WorthQueryBranchSetAdoptionResumeDenial::Preparation(
                    WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                ),
                retained.expect("refusal precedes admission"),
            )),
        }
    }
}

fn resume_failure(
    denial: WorthQueryBranchSetAdoptionResumeDenial,
    adoption: WorthQueryStoppedBranchSetAdoption,
) -> WorthQueryBranchSetAdoptionResumeFailure {
    WorthQueryBranchSetAdoptionResumeFailure { denial, adoption }
}
