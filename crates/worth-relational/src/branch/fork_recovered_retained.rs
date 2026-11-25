//! Replay one recorded retained fork from its durably rebuilt historical root.
//!
//! Recovery resolves the recorded `source_head` commit to the exact immutable
//! root it selected, verifies the root against that head, and then runs the
//! same fork kernel a live retained fork uses. The replay step requires the
//! kernel to reproduce the recorded transition exactly, so a source that is no
//! longer live, a target that exists or is retired, or a root that does not
//! match the recorded head refuses before any effect.
use super::source_validation::{ForkSourceCustody, ForkSourceInput, ValidatedForkSource};
use super::{RelationalForkDenial, RelationalForkOutcome};
use crate::branch::fork_source_basis::RelationalForkSourceDescriptor;
use crate::branch::transition::BranchTransitionRecordError;
use crate::branch::{
    RelationalBranchRoot, RelationalBranchTransitionDurableStep, RelationalBranchVersion,
    RelationalForkPort,
};
use crate::history::data::BranchId;
use std::sync::Arc;
use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};
use worth_relational_branch::internal_port::branch::basis_axis_validation::require_root_matches_reference;
use worth_relational_branch::internal_port::branch::fork_source_basis::RelationalForkSourceDescriptorOwnerPort as _;
use worth_relational_branch::internal_port::branch::reference::relational_branch_observation;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTarget;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

/// The recorded axes of one retained fork plus its rebuilt historical root.
pub(crate) struct RecordedRetainedForkSource<'a> {
    pub(crate) branch: &'a BranchId,
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    /// The recorded head in the canonical runtime frame.
    pub(crate) head: &'a RelationalBranchTarget,
    pub(crate) root: Arc<RelationalBranchRoot>,
}

impl RelationalForkPort {
    /// Replay a recorded retained fork through `step`.
    ///
    /// Cost: the fork kernel's own cost; O(1) beyond it.
    pub(crate) fn fork_recovered_retained_with<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        target_branch: BranchId,
        source: RecordedRetainedForkSource<'_>,
    ) -> Result<RelationalForkOutcome, BranchTransitionRecordError<RelationalForkDenial, S::Stop>>
    {
        let denied = BranchTransitionRecordError::Denied;
        let validated = self.recovered_retained_source(source).map_err(denied)?;
        let reservation = self.reserve_fork_target(target_branch).map_err(denied)?;
        self.fork_reserved_input_with_hooks(
            execution,
            step,
            reservation,
            ForkSourceInput::Recovered(Box::new(validated)),
            || {},
            || {},
            false,
        )
        .map(|installed| installed.outcome)
    }

    fn recovered_retained_source(
        &self,
        source: RecordedRetainedForkSource<'_>,
    ) -> Result<ValidatedForkSource, RelationalForkDenial> {
        if self.owner.branch_cell(source.branch).is_none() {
            return Err(RelationalForkDenial::SourceBranchMissing);
        }
        let observation = relational_branch_observation(
            self.runtime_instance_id,
            &source.branch.0,
            FoundationalBranchTarget::basis(
                source
                    .head
                    .rebind_runtime_instance_id(self.runtime_instance_id),
            ),
            source.generation,
        )
        .map_err(|_| RelationalForkDenial::EmptySource)?;
        require_root_matches_reference(&observation, &source.root)
            .map_err(|_| RelationalForkDenial::MissingArtifact)?;
        Ok(ValidatedForkSource {
            descriptor: RelationalForkSourceDescriptor::new(
                self.runtime_instance_id,
                observation.clone(),
                source.branch.clone(),
                source.truth_version,
            ),
            source_observation: observation,
            source_truth_version: source.truth_version,
            source_root: source.root,
            source_head: source.head.clone(),
            custody: ForkSourceCustody::Recovered,
        })
    }
}
