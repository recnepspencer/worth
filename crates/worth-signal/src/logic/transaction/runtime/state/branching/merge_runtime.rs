mod artifact_projection;
mod aspect_policy;
mod aspect_policy_selection;
mod candidates;
mod conflict_classification;
mod conflict_isolation;
mod conflict_policy;
mod conflict_resolution;
mod correspondence;
mod correspondence_evidence;
mod deletion_policy;
mod execution_application;
mod execution_artifacts;
mod execution_finalization;
mod execution_lifecycle;
mod execution_preparation;
mod execution_summary;
mod identity_matcher;
mod merge_base_strategy;
mod node_plan;
mod plan_compiler;
mod request_boundary;
mod result_projection;
mod source_only_policy;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchMergeDenial,
    SignalBranchMergeOutcome,
};
use crate::data::error::SignalError;
use crate::state::SignalBranchHandle;

use super::super::merge::{
    BranchMergePlan, BranchMergeRequest, BranchMergeRequestDenial, BranchMergeResult,
    LoweredFoundationalMergeRequest,
};

#[cfg(test)]
use super::super::merge::BranchMergeExecutionSummary;
use super::super::runtime_state::SignalRuntime;
use super::branches::SignalBranchSnapshotStorageDenial;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn merge_branch(
        &mut self,
        source: &AdmittedSignalBranchBasis,
        target: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchMergeOutcome, SignalBranchMergeDenial> {
        let (source, target, retention) = self.preflight_signal_branch_merge(source, target)?;
        let result = self
            .merge_branch_raw(source, target.clone())
            .map_err(|error| SignalBranchMergeDenial::OwnerFailed { error })?;
        let target_basis = self
            .admit_signal_branch_with_retention(target, retention)
            .map_err(|error| SignalBranchMergeDenial::OwnerFailed { error })?;
        Ok(SignalBranchMergeOutcome::owner_issued(target_basis, result))
    }

    pub(crate) fn merge_branch_raw(
        &mut self,
        source: SignalBranchHandle,
        target: SignalBranchHandle,
    ) -> Result<BranchMergeResult, SignalError> {
        let normalized_request = BranchMergeRequest::full_branch(source, target)
            .normalize()
            .map_err(BranchMergeRequestDenial::into_signal_error)?;
        let lowered_request = self.lower_foundational_merge_request(&normalized_request)?;
        let plan = self.plan_branch_merge_request(&lowered_request)?;
        self.execute_branch_merge_request_plan(&lowered_request, &plan)
    }

    pub(crate) fn validate_signal_branch_merge_bases(
        &self,
        source: &AdmittedSignalBranchBasis,
        target: &AdmittedSignalBranchBasis,
    ) -> Result<(SignalBranchHandle, SignalBranchHandle), SignalBranchMergeDenial> {
        let source_id = source.owner_branch_id();
        let source_branch = self.branches.branch_handle(source_id).ok_or(
            SignalBranchMergeDenial::UnknownSourceBranch {
                branch_id: source_id,
            },
        )?;
        let source_live = self
            .signal_branch_observation(&source_branch)
            .map_err(|_| SignalBranchMergeDenial::UnknownSourceBranch {
                branch_id: source_id,
            })?;
        if let Err(mismatch) = source_live.compare(source.observation()) {
            return Err(SignalBranchMergeDenial::SourceBasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }

        let target_id = target.owner_branch_id();
        let target_branch = self.branches.branch_handle(target_id).ok_or(
            SignalBranchMergeDenial::UnknownTargetBranch {
                branch_id: target_id,
            },
        )?;
        let target_live = self
            .signal_branch_observation(&target_branch)
            .map_err(|_| SignalBranchMergeDenial::UnknownTargetBranch {
                branch_id: target_id,
            })?;
        if let Err(mismatch) = target_live.compare(target.observation()) {
            return Err(SignalBranchMergeDenial::TargetBasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        Ok((source_branch, target_branch))
    }

    fn preflight_signal_branch_merge(
        &mut self,
        source: &AdmittedSignalBranchBasis,
        target: &AdmittedSignalBranchBasis,
    ) -> Result<
        (
            SignalBranchHandle,
            SignalBranchHandle,
            SignalBranchAdmissionLease,
        ),
        SignalBranchMergeDenial,
    > {
        let (source, target) = self.validate_signal_branch_merge_bases(source, target)?;
        self.branches
            .ensure_snapshot_storage_available()
            .map_err(|denial| match denial {
                SignalBranchSnapshotStorageDenial::CapacityExhausted {
                    maximum_stored_snapshots,
                } => SignalBranchMergeDenial::SnapshotCapacityExhausted {
                    maximum_stored_snapshots,
                },
            })?;
        let next_snapshot_id = self
            .branches
            .replay_graph(target.id, self.graph.current_branch().id, &self.graph)
            .expect("validated merge target retains live graph state")
            .diagnostics_state()
            .branch_snapshot_allocator_state()
            .0;
        self.branches
            .synchronize_snapshot_identity_high_water(next_snapshot_id);
        self.branches
            .snapshot_identity_available()
            .map_err(
                |next_snapshot_id| SignalBranchMergeDenial::SnapshotIdentityExhausted {
                    next_snapshot_id,
                },
            )?;
        let retention = self
            .branches
            .acquire_admitted_retention(target.id)
            .map_err(|denial| SignalBranchMergeDenial::RetentionUnavailable { denial })?;
        Ok((source, target, retention))
    }

    pub(crate) fn execute_admitted_branch_merge_request_plan(
        &mut self,
        source_basis: &AdmittedSignalBranchBasis,
        target_basis: &AdmittedSignalBranchBasis,
        request: &LoweredFoundationalMergeRequest,
        plan: &BranchMergePlan,
    ) -> Result<SignalBranchMergeOutcome, SignalError> {
        let (_, target, retention) = self
            .preflight_signal_branch_merge(source_basis, target_basis)
            .map_err(signal_branch_merge_denial_to_error)?;
        let result = self.execute_branch_merge_request_plan(request, plan)?;
        let target_basis = self.admit_signal_branch_with_retention(target, retention)?;
        Ok(SignalBranchMergeOutcome::owner_issued(target_basis, result))
    }

    pub(crate) fn lower_foundational_merge_request(
        &mut self,
        request: &crate::logic::transaction::runtime::NormalizedBranchMergeRequest,
    ) -> Result<LoweredFoundationalMergeRequest, SignalError> {
        request_boundary::lower_foundational_request(self, request)
    }

    pub(crate) fn plan_branch_merge_request(
        &mut self,
        request: &LoweredFoundationalMergeRequest,
    ) -> Result<BranchMergePlan, SignalError> {
        request_boundary::admit_and_compile(self, request)
    }

    pub(crate) fn execute_branch_merge_request_plan(
        &mut self,
        request: &LoweredFoundationalMergeRequest,
        plan: &BranchMergePlan,
    ) -> Result<BranchMergeResult, SignalError> {
        execution_lifecycle::execute_and_project(self, request, plan)
    }

    #[cfg(test)]
    pub(crate) fn execute_branch_merge_request_plan_summary_for_test(
        &mut self,
        request: &LoweredFoundationalMergeRequest,
        plan: &BranchMergePlan,
    ) -> Result<BranchMergeExecutionSummary, SignalError> {
        execution_lifecycle::execute_summary_for_test(self, request, plan)
    }

    #[cfg(test)]
    pub(crate) fn inspect_branch_merge_plan_for_test(
        &mut self,
        source: SignalBranchHandle,
        target: SignalBranchHandle,
    ) -> Result<BranchMergePlan, SignalError> {
        let normalized_request = BranchMergeRequest::full_branch(source, target)
            .normalize()
            .map_err(BranchMergeRequestDenial::into_signal_error)?;
        let lowered_request = self.lower_foundational_merge_request(&normalized_request)?;
        self.plan_branch_merge_request(&lowered_request)
    }
}

fn signal_branch_merge_denial_to_error(denial: SignalBranchMergeDenial) -> SignalError {
    SignalError::invalid_input(format!("canonical Signal branch merge denied: {denial:?}"))
}
