use crate::history::data::commit_receipt::RelationalCommitReceipt;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::snapshots::data::SnapshotHandle;

#[derive(Debug, Clone)]
pub(crate) struct PublicationArtifacts {
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) snapshot: SnapshotHandle,
    pub(crate) diagnostics_summary: crate::diagnostics::data::RelationalDiagnosticArtifact,
    pub(crate) patch: CanonicalAuthoritativePatch,
    pub(crate) schema_authority: crate::schema::data::SchemaAuthoritySnapshot,
}
