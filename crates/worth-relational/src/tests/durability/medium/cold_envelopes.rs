//! A bounded history operation demands its selected journal envelope only.
use crate::execution::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug)]
pub(super) enum EnvelopeSurface {
    VersionSummary,
    CommitInspection,
    RecentInspection,
    RecentBranchInspection,
    CompiledArtifact,
}

impl EnvelopeSurface {
    pub(super) const ALL: [Self; 5] = [
        Self::VersionSummary,
        Self::CommitInspection,
        Self::RecentInspection,
        Self::RecentBranchInspection,
        Self::CompiledArtifact,
    ];
    pub(super) fn read(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        receipt: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<usize, RelationalExecutionDenialCause> {
        Ok(match self {
            Self::CompiledArtifact => {
                runtime
                    .compiled_artifacts_authority()
                    .compile_execution_artifact(
                        execution,
                        receipt.commit_id,
                        vec![crate::identity::data::PartitionId::main()],
                    )
                    .map_err(|failure| match failure.authority_status {
                        crate::simulation::data::CompiledArtifactAuthorityStatus::ReadRequest(
                            cause,
                        ) => cause,
                        other => {
                            panic!("enabled fixture compile source must be readable: {other:?}")
                        }
                    })?
                    .compiled_record_count
            }
            Self::VersionSummary => runtime
                .history()
                .historical_committed_version(execution, receipt.version_id)?
                .expect("fixture's indexed version exists")
                .changed_record_count(),
            Self::CommitInspection => runtime
                .inspect_what_happened()
                .inspect_commit(execution, receipt.commit_id)?
                .expect("fixture's indexed commit exists")
                .changed_records
                .len(),
            Self::RecentBranchInspection => runtime
                .inspect_what_happened()
                .inspect_recent_commits(
                    execution,
                    &crate::inspection::data::RecentCommitInspectionRequest {
                        branch_id: Some(receipt.branch_id.clone()),
                        limit: 2,
                    },
                )?
                .commits
                .len(),
            Self::RecentInspection => runtime
                .inspect_what_happened()
                .inspect_recent_commits(
                    execution,
                    &crate::inspection::data::RecentCommitInspectionRequest {
                        branch_id: None,
                        limit: 2,
                    },
                )?
                .commits
                .len(),
        })
    }
}
