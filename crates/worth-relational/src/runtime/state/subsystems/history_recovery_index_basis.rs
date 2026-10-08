//! Join a retained scoped cache to already readmitted native history authority.
use worth_foundational::FoundationalBranchTarget;

use crate::branch::RelationalBranchTarget;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;

use super::{history_recovery_lineage::validate_branch_target_lineage, HistorySubsystem};

impl HistorySubsystem {
    /// A cache may lag its observing branch. Its immutable basis must still
    /// belong to that branch's own stream or exact admitted fork provenance.
    /// The caller has already validated the recovered branch cells and roots.
    pub(crate) fn validate_recovered_scoped_index_basis(
        &self,
        branch: &BranchId,
        commit: CommitId,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Result<(), String> {
        let cell = self.branch_cell(branch).ok_or_else(|| {
            format!(
                "checkpoint scoped index names unissued branch `{}`",
                branch.0
            )
        })?;
        let artifact = self.commit_artifact(commit).ok_or_else(|| {
            format!(
                "checkpoint scoped index names missing commit `{}`",
                commit.0
            )
        })?;
        if artifact.version_id() != version || artifact.envelope().schema_version != schema {
            return Err("checkpoint scoped index basis mismatches admitted commit".to_owned());
        }
        let target = FoundationalBranchTarget::basis(RelationalBranchTarget::from_commit_receipt(
            self.runtime_instance_id,
            &artifact.envelope().commit,
            artifact.roots().clone(),
        ));
        validate_branch_target_lineage(
            self,
            branch,
            &target,
            cell.fork_source_branch_id().as_ref(),
            cell.fork_provenance().as_ref(),
        )
    }
}
