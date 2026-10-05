use serde::Deserialize;

use super::lineage::LegacyPublishedLineageArtifact;
use super::schema_authority::LegacySchemaAuthoritySnapshot;
use crate::history::data::{
    BranchId, CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, CommitId,
};
use crate::publication::patch::data::{CanonicalAuthoritativePatch, PatchStreamPosition};

#[derive(Deserialize)]
struct LegacyDurableSegmentFile {
    entries: Vec<LegacyCanonicalCommitEnvelope>,
}

pub(crate) enum LegacySegmentDecodeError {
    Decode,
    Schema(String),
    UnsupportedLineage(String),
}

#[derive(Deserialize)]
struct LegacyCanonicalCommitEnvelope {
    commit: crate::history::data::RelationalCommitReceipt,
    branch_context: BranchId,
    authority_kind: LegacyCanonicalCommitAuthorityKind,
    strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    merge_execution_authority: Option<crate::transactions::data::PublishedMergeExecutionAuthority>,
    merge_parent_branches: Vec<BranchId>,
    merge_base_commits: Vec<CommitId>,
    schema_version: crate::schema::data::SchemaVersionId,
    schema_authority: LegacySchemaAuthoritySnapshot,
    merged_plan: crate::transactions::data::MergedCommitPlan,
    patch: LegacyPositionedPatch,
    diagnostics_summary: crate::diagnostics::data::RelationalDiagnosticArtifact,
    lineage: LegacyPublishedLineageArtifact,
    derived_index_artifacts: crate::indexes::data::DerivedIndexArtifacts,
    schema_transition: Option<crate::schema::data::SchemaTransitionArtifact>,
    schema_continuation_descriptor: Option<crate::schema::data::SchemaContinuationDescriptor>,
    schema_reconciliation_descriptor: Option<crate::schema::data::SchemaReconciliationDescriptor>,
    descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
}

#[derive(Deserialize)]
enum LegacyCanonicalCommitAuthorityKind {
    VersionedTransaction,
    MetadataOnlyLineage,
}

#[derive(Deserialize)]
struct LegacyPositionedPatch {
    ordering: crate::publication::patch::data::PatchOrdering,
    publication_mode: crate::publication::patch::data::PatchPublicationMode,
    position: PatchStreamPosition,
    authoritative_record_patches:
        Vec<crate::publication::patch::data::PublishedAuthoritativeRecordPatch>,
}

pub(crate) fn decode_segment(
    bytes: &[u8],
    registry: &crate::schema::data::RelationalSchemaRegistry,
) -> Result<Vec<crate::durability::migration::ReadmittedCanonicalCommit>, LegacySegmentDecodeError>
{
    let file = rmp_serde::from_slice::<LegacyDurableSegmentFile>(bytes)
        .map_err(|_| LegacySegmentDecodeError::Decode)?;
    file.entries
        .into_iter()
        .map(|entry| entry.readmit(registry))
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn segment_inventory(
    bytes: &[u8],
) -> Result<Vec<crate::history::data::RelationalCommitReceipt>, String> {
    rmp_serde::from_slice::<LegacyDurableSegmentFile>(bytes)
        .map(|file| file.entries.into_iter().map(|entry| entry.commit).collect())
        .map_err(|error| error.to_string())
}

impl LegacyCanonicalCommitEnvelope {
    fn readmit(
        self,
        registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::durability::migration::ReadmittedCanonicalCommit, LegacySegmentDecodeError>
    {
        let position = self.patch.position;
        let schema_authority = self
            .schema_authority
            .readmit(registry)
            .map_err(LegacySegmentDecodeError::Schema)?;
        let metadata_only_lineage = matches!(
            self.authority_kind,
            LegacyCanonicalCommitAuthorityKind::MetadataOnlyLineage
        );
        let (lineage, legacy_lineage_provenance) = self
            .lineage
            .readmit(&self.commit, metadata_only_lineage)
            .map_err(|error| LegacySegmentDecodeError::UnsupportedLineage(error.detail()))?;
        if !legacy_lineage_provenance.validates_translation(&lineage) {
            return Err(LegacySegmentDecodeError::UnsupportedLineage(
                "legacy lineage readmission did not preserve correspondence semantics".into(),
            ));
        }
        let canonical = CanonicalCommitEnvelope::new(
            self.commit,
            self.branch_context,
            if metadata_only_lineage {
                CanonicalCommitAuthorityKind::BranchReferenceMovement
            } else {
                CanonicalCommitAuthorityKind::VersionedTransaction
            },
            self.strategy_artifacts,
            self.merge_execution_authority,
            self.merge_parent_branches,
            self.merge_base_commits,
            self.schema_version,
            schema_authority,
            self.merged_plan,
            CanonicalAuthoritativePatch {
                ordering: self.patch.ordering,
                publication_mode: self.patch.publication_mode,
                authoritative_record_patches: self.patch.authoritative_record_patches,
            },
            crate::history::data::RelationalDescriptiveTouchGraph::unavailable(),
            self.diagnostics_summary,
            lineage,
            self.derived_index_artifacts,
            self.schema_transition,
            self.schema_continuation_descriptor,
            self.schema_reconciliation_descriptor,
            self.descriptor_semantics_version,
        );
        Ok(
            crate::durability::migration::ReadmittedCanonicalCommit::requires_replay_completion(
                position, canonical,
            ),
        )
    }
}
