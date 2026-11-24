use super::MergeAccess;
use crate::merge::data::{
    MergePlanningArtifactCore, PreparedMergeExecution, RelationalMergeStrategyWitness,
};

impl<'runtime> MergeAccess<'runtime> {
    pub fn retain_merge_strategy_witness_from_prepared_execution(
        &self,
        prepared: &PreparedMergeExecution,
    ) -> RelationalMergeStrategyWitness {
        prepared.artifact().strategy_witness.clone()
    }

    pub fn retain_merge_strategy_witness_from_planning_artifact(
        &self,
        artifact: &MergePlanningArtifactCore,
    ) -> RelationalMergeStrategyWitness {
        artifact.strategy_witness.clone()
    }
}
