use super::MergeAccess;
use crate::merge::data::{
    MergePlanningArtifactCore, PreparedMergeExecution, RelationalSchemaReconciliationWitness,
};

impl<'runtime> MergeAccess<'runtime> {
    pub fn retain_schema_reconciliation_witness_from_prepared_execution(
        &self,
        prepared: &PreparedMergeExecution,
    ) -> RelationalSchemaReconciliationWitness {
        prepared.artifact().schema_reconciliation_witness.clone()
    }

    pub fn retain_schema_reconciliation_witness_from_planning_artifact(
        &self,
        artifact: &MergePlanningArtifactCore,
    ) -> RelationalSchemaReconciliationWitness {
        artifact.schema_reconciliation_witness.clone()
    }
}
