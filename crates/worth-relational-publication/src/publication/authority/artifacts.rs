use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch;
use worth_relational_branch::internal_port::snapshots::data::SnapshotHandle;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Clone)]
pub(crate) struct PublicationArtifacts {
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) snapshot: SnapshotHandle,
    pub(crate) diagnostics_summary:
        contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact,
    pub(crate) patch: CanonicalAuthoritativePatch,
    pub(crate) schema_authority:
        contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot,
}
