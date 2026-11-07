mod bridge;
mod bridge_oracle;
mod error;
mod relational;
mod verification;

use super::batch_execution::ExecutedEffectBatchPlan;
use super::execution::ExecutedEffectPlan;
use super::execution_relational_scalar::mutation_target_branch;
use crate::WorthQueryEvidenceIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

#[cfg(test)]
pub(crate) use bridge_oracle::bridge_observation_execution_receipt_subject_identity;
pub use bridge_oracle::{
    bridge_observation_execution_record_subject_identity,
    bridge_observation_outcome_subject_identity, bridge_observation_receipt_subject_identity,
    bridge_observation_request_subject_identity, BridgeExecutionOracle,
};
pub use error::{EffectExecutionOracleError, EffectExecutionOracleErrorKind};
pub use relational::RelationalExecutionOracle;
pub use verification::{EffectExecutionOracleVerification, EffectExecutionOracleVerificationKind};

impl ExecutedEffectPlan {
    pub fn verify_against_relational_runtime(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let target_branch = relational_target_branch_for_plan(self)?;
        let oracle = RelationalExecutionOracle::observe_branch_head(runtime, &target_branch)?;
        self.verify_against_relational_oracle(&oracle)
    }

    pub fn verify_against_relational_oracle(
        &self,
        oracle: &RelationalExecutionOracle,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let (
            expected_branch,
            observed_commit_id,
            observed_version_id,
            observed_parent_commit_ids,
            verification_kind,
        ) = relational_subject_for_plan(self)?;
        verify_relational_subject(
            self.effect_execution_identity(),
            expected_branch,
            observed_commit_id,
            observed_version_id,
            observed_parent_commit_ids,
            verification_kind,
            oracle,
            1,
        )
    }
}

impl ExecutedEffectBatchPlan {
    pub fn verify_against_relational_runtime(
        &self,
        runtime: &RelationalRuntime,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let target_branch = relational_target_branch_for_batch(self)?;
        let oracle = RelationalExecutionOracle::observe_branch_head(runtime, &target_branch)?;
        self.verify_against_relational_oracle(&oracle)
    }

    pub fn verify_against_relational_oracle(
        &self,
        oracle: &RelationalExecutionOracle,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let target_branch = relational_target_branch_for_batch(self)?;
        let aggregate = self.aggregate_mutation().ok_or_else(|| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
                "relational batch oracle verification currently requires an aggregate mutation artifact",
                self.batch_identity(),
                Some(oracle.relational_oracle_identity()),
            )
        })?;
        verify_relational_subject(
            self.batch_identity(),
            target_branch.0,
            aggregate.outcome().commit.commit_id.0,
            aggregate.outcome().commit.version_id.0,
            aggregate
                .outcome()
                .commit
                .parents
                .iter()
                .map(|parent| parent.0)
                .collect(),
            EffectExecutionOracleVerificationKind::MutationBatch,
            oracle,
            self.components().len(),
        )
    }
}

fn relational_subject_for_plan(
    executed: &ExecutedEffectPlan,
) -> Result<
    (
        String,
        u64,
        u64,
        Vec<u64>,
        EffectExecutionOracleVerificationKind,
    ),
    EffectExecutionOracleError,
> {
    if let Some(mutation) = executed.lowered().as_mutation() {
        let branch = mutation_target_branch(mutation).map_err(|(_, message)| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
                message,
                executed.effect_execution_identity(),
                None,
            )
        })?;
        let commit = executed
            .as_mutation()
            .expect("mutation lowering should retain mutation artifact");
        return Ok((
            branch.0,
            commit.outcome().commit.commit_id.0,
            commit.outcome().commit.version_id.0,
            commit
                .outcome()
                .commit
                .parents
                .iter()
                .map(|parent| parent.0)
                .collect(),
            EffectExecutionOracleVerificationKind::Mutation,
        ));
    }
    if let Some(merge) = executed.lowered().as_merge() {
        let outcome = executed
            .as_merge()
            .expect("merge lowering should retain merge artifact");
        return Ok((
            merge.merge_request().target_branch().0.clone(),
            outcome.commit.outcome().commit.commit_id.0,
            outcome.commit.outcome().commit.version_id.0,
            outcome
                .commit
                .outcome()
                .commit
                .parents
                .iter()
                .map(|parent| parent.0)
                .collect(),
            EffectExecutionOracleVerificationKind::Merge,
        ));
    }
    Err(EffectExecutionOracleError::new(
        EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
        "relational oracle verification requires an executed mutation or merge artifact",
        executed.effect_execution_identity(),
        None,
    ))
}

fn relational_target_branch_for_plan(
    executed: &ExecutedEffectPlan,
) -> Result<BranchId, EffectExecutionOracleError> {
    if let Some(mutation) = executed.lowered().as_mutation() {
        return mutation_target_branch(mutation).map_err(|(_, message)| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
                message,
                executed.effect_execution_identity(),
                None,
            )
        });
    }
    if let Some(merge) = executed.lowered().as_merge() {
        return Ok(merge.merge_request().target_branch().clone());
    }
    Err(EffectExecutionOracleError::new(
        EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
        "relational oracle verification requires an executed mutation or merge artifact",
        executed.effect_execution_identity(),
        None,
    ))
}

fn relational_target_branch_for_batch(
    executed: &ExecutedEffectBatchPlan,
) -> Result<BranchId, EffectExecutionOracleError> {
    let mut branches = executed
        .components()
        .iter()
        .map(relational_target_branch_for_plan)
        .collect::<Result<Vec<_>, _>>()?;
    let Some(first) = branches.pop() else {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::RelationalOracleUnsupportedEffect,
            "relational batch oracle verification requires at least one executed component",
            executed.batch_identity(),
            None,
        ));
    };
    if branches.iter().any(|branch| branch != &first) {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BatchOracleMixedTargetBranch,
            "relational batch oracle verification requires one target branch across all components",
            executed.batch_identity(),
            None,
        ));
    }
    Ok(first)
}

fn verify_relational_subject(
    execution_subject_identity: &WorthQueryEvidenceIdentity,
    expected_branch: String,
    observed_commit_id: u64,
    observed_version_id: u64,
    observed_parent_commit_ids: Vec<u64>,
    verification_kind: EffectExecutionOracleVerificationKind,
    oracle: &RelationalExecutionOracle,
    component_count: usize,
) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
    if oracle.branch_identity() != expected_branch {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::RelationalOracleBranchMismatch,
            format!(
                "relational oracle observed branch `{}` but lowered execution targets `{expected_branch}`",
                oracle.branch_identity()
            ),
            execution_subject_identity,
            Some(oracle.relational_oracle_identity()),
        ));
    }
    if oracle.observed_commit_id() != observed_commit_id
        || oracle.observed_version_id() != observed_version_id
    {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::RelationalOracleCommitMismatch,
            format!(
                "relational oracle observed commit/version `{}:{}` but executed artifact produced `{}:{}`",
                oracle.observed_commit_id(),
                oracle.observed_version_id(),
                observed_commit_id,
                observed_version_id
            ),
            execution_subject_identity,
            Some(oracle.relational_oracle_identity()),
        ));
    }
    if oracle.observed_parent_commit_ids() != observed_parent_commit_ids.as_slice() {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::RelationalOracleCommitMismatch,
            format!(
                "relational oracle observed parent topology `{:?}` but executed artifact produced `{:?}`",
                oracle.observed_parent_commit_ids(),
                observed_parent_commit_ids
            ),
            execution_subject_identity,
            Some(oracle.relational_oracle_identity()),
        ));
    }
    Ok(EffectExecutionOracleVerification::relational(
        verification_kind,
        execution_subject_identity,
        oracle,
        component_count,
    ))
}
