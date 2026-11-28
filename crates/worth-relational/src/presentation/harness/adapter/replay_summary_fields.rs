use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::PatchStreamPosition;
use crate::snapshots::data::SnapshotId;
use super::terminal_harness_summary_projection::optional_terminal_harness_summary_projection_u64;

use super::terminal_harness_summary_projection::{
    terminal_harness_summary_projection_object, terminal_harness_summary_projection_string,
    terminal_harness_summary_projection_u64, terminal_harness_summary_projection_usize,
    TerminalHarnessSummaryProjection,
};

pub(super) fn replay_summary(
    envelope: &CanonicalCommitEnvelope,
    position: PatchStreamPosition,
    snapshot_id: Option<SnapshotId>,
) -> TerminalHarnessSummaryProjection {
    ReplaySummary::from_envelope(envelope, position, snapshot_id).into_terminal_harness_summary_projection()
}

struct ReplaySummary {
    schema_version: u64,
    commit_id: u64,
    version_id: u64,
    snapshot_id: Option<u64>,
    patch_stream_position: u64,
    patch_record_count: usize,
    patch_ordering: String,
    patch_publication_mode: String,
}

impl ReplaySummary {
    fn from_envelope(envelope: &CanonicalCommitEnvelope, position: PatchStreamPosition, snapshot_id: Option<SnapshotId>) -> Self {
        Self {
            schema_version: 1,
            commit_id: envelope.commit.commit_id.0,
            version_id: envelope.commit.version_id.0,
            snapshot_id: snapshot_id.map(|id| id.0),
            patch_stream_position: position.0,
            patch_record_count: envelope.patch.authoritative_record_patches.len(),
            patch_ordering: format!("{:?}", envelope.patch.ordering),
            patch_publication_mode: format!("{:?}", envelope.patch.publication_mode),
        }
    }

    fn into_terminal_harness_summary_projection(self) -> TerminalHarnessSummaryProjection {
        terminal_harness_summary_projection_object([
            (
                "schema_version",
                terminal_harness_summary_projection_u64(self.schema_version),
            ),
            (
                "commit_id",
                terminal_harness_summary_projection_u64(self.commit_id),
            ),
            (
                "version_id",
                terminal_harness_summary_projection_u64(self.version_id),
            ),
            (
                "snapshot_id",
                optional_terminal_harness_summary_projection_u64(self.snapshot_id),
            ),
            (
                "patch_stream_position",
                terminal_harness_summary_projection_u64(self.patch_stream_position),
            ),
            (
                "patch_record_count",
                terminal_harness_summary_projection_usize(self.patch_record_count),
            ),
            (
                "patch_ordering",
                terminal_harness_summary_projection_string(self.patch_ordering),
            ),
            (
                "patch_publication_mode",
                terminal_harness_summary_projection_string(self.patch_publication_mode),
            ),
        ])
    }
}
