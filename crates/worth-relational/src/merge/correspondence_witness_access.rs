use super::MergeAccess;
use crate::merge::data::{
    MergePlanningArtifactCore, PreparedMergeExecution, RelationalMergeCorrespondenceWitness,
};

impl<'runtime> MergeAccess<'runtime> {
    pub fn retain_merge_correspondence_witness_from_prepared_execution(
        &self,
        prepared: &PreparedMergeExecution,
    ) -> RelationalMergeCorrespondenceWitness {
        self.retain_merge_correspondence_witness_from_planning_artifact(prepared.artifact())
    }

    pub fn retain_merge_correspondence_witness_from_planning_artifact(
        &self,
        artifact: &MergePlanningArtifactCore,
    ) -> RelationalMergeCorrespondenceWitness {
        super::correspondence_witness::retain_merge_correspondence_witness_from_planning_artifact(
            artifact,
        )
    }
}
