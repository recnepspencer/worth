use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::publication::bundle::publication_status::PublicationStatus;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelope;
use worth_relational_branch::internal_port::snapshots::data::SnapshotHandle;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationBundle<ReplayRecord> {
    pub commit: RelationalCommitReceipt,
    pub snapshot: SnapshotHandle,
    pub diagnostics_summary: RelationalDiagnosticArtifact,
    pub patch: PublishedAuthoritativePatchEnvelope,
    pub replay: ReplayRecord,
    pub status: PublicationStatus,
}
