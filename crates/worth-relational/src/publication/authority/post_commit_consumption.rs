use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::history::data::{BranchId, CommitId};
use crate::publication::authority::post_commit_diagnostics::build_publication_diagnostic_entries;
use crate::publication::PublicationAuthority;
use crate::snapshots::data::SnapshotId;

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn consume_post_commit_artifacts(
        &self,
        commit_id: CommitId,
        snapshot_id: SnapshotId,
        branch_id: BranchId,
        parents: &[CommitId],
        merge_parent_branches: &[BranchId],
        merge_base_commits: &[CommitId],
    ) {
        const PACKET_COUNT: usize = 1;
        self.runtime
            .performance_access()
            .count_post_commit_consumer_shape(PACKET_COUNT, PACKET_COUNT, 1, 1);

        self.runtime
            .performance_access()
            .count_post_commit_serial_strategy();

        let consumption_context =
            super::post_commit_consumer::PostCommitConsumptionContext::new(commit_id, snapshot_id);
        let consumer_failure = self
            .runtime
            .publication
            .post_commit_consumer
            .consume(&consumption_context)
            .err();
        let entries = build_publication_diagnostic_entries(
            commit_id,
            snapshot_id,
            &branch_id,
            parents,
            merge_parent_branches,
            merge_base_commits,
            consumer_failure,
        );
        self.push_bounded_diagnostic(
            DiagnosticsScope::PatchPublication,
            DiagnosticsArtifactKind::MinimalSummary,
            entries,
        );
    }
}
