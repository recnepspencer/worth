use crate::publication::authority::post_commit_diagnostics::build_publication_diagnostic_entries;
use crate::publication::authority::PublicationAuthority;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::snapshots::data::coordinates::SnapshotId;
use worth_relational_contracts::internal_port as contracts;

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn consume_post_commit_artifacts(
        &self,
        commit_id: CommitId,
        snapshot_id: SnapshotId,
        branch_id: BranchId,
        parents: &[CommitId],
    ) {
        self.instrumentation.count(|counters| {
            counters.post_commit_consumer_packet_count += 1;
            counters.post_commit_consumer_item_count += 1;
            counters.post_commit_consumer_peak_width_total += 1;
            counters.post_commit_scope_unit_count += 1;
            counters.post_commit_serial_strategy_count += 1;
        });

        let consumption_context =
            super::post_commit_consumer::PostCommitConsumptionContext::new(commit_id, snapshot_id);
        let consumer_failure = self
            .publication
            .post_commit_consumer
            .consume(&consumption_context)
            .err();
        let entries = build_publication_diagnostic_entries(
            commit_id,
            snapshot_id,
            &branch_id,
            parents,
            &[],
            &[],
            consumer_failure,
        );
        self.push_bounded_diagnostic(
            DiagnosticsScope::PatchPublication,
            DiagnosticsArtifactKind::MinimalSummary,
            entries,
        );
    }
}
