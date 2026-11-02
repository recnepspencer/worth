//! The private mutable preparation phase cannot encode an incomplete envelope.
use super::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::{
    BranchId, CanonicalAuthoredIndexWrites, CanonicalCommitAuthorityKind, CommitId,
    RelationalCommitReceipt, RelationalDescriptiveTouchGraph,
};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::PublishedLineageArtifact;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaAuthoritySnapshot, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor, SchemaTransitionArtifact, SchemaVersionId,
};
use crate::transactions::data::{MergedCommitPlan, PublishedMergeExecutionAuthority};

pub(crate) struct CanonicalCommitEnvelopeDraft {
    body: CanonicalCommitEnvelopeBody,
}
impl std::ops::Deref for CanonicalCommitEnvelopeDraft {
    type Target = CanonicalCommitEnvelopeBody;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}
impl std::ops::DerefMut for CanonicalCommitEnvelopeDraft {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}
impl CanonicalCommitEnvelopeDraft {
    pub(crate) fn new(
        index_definition_delta: super::CanonicalIndexDefinitionDelta,
        commit: RelationalCommitReceipt,
        branch_context: BranchId,
        authority_kind: CanonicalCommitAuthorityKind,
        strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
        merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
        merge_parent_branches: Vec<BranchId>,
        merge_base_commits: Vec<CommitId>,
        schema_version: SchemaVersionId,
        schema_authority: SchemaAuthoritySnapshot,
        merged_plan: MergedCommitPlan,
        patch: CanonicalAuthoritativePatch,
        descriptive_touches: RelationalDescriptiveTouchGraph,
        diagnostics_summary: RelationalDiagnosticArtifact,
        lineage: PublishedLineageArtifact,
        derived_index_artifacts: DerivedIndexArtifacts,
        schema_transition: Option<SchemaTransitionArtifact>,
        schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
        schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
        descriptor_semantics_version: DescriptorSemanticsVersion,
    ) -> Self {
        Self {
            body: CanonicalCommitEnvelopeBody {
                index_definition_delta,
                caller_association: None,
                authored_index_writes: CanonicalAuthoredIndexWrites::default(),
                required_output_codec_retirements: Vec::new(),
                required_output_origin: None,
                required_output_materialization: None,
                commit,
                branch_context,
                branch_cell_checkpoint: None,
                authority_kind,
                strategy_artifacts,
                merge_execution_authority,
                merge_parent_branches,
                merge_base_commits,
                schema_version,
                schema_authority,
                merged_plan,
                record_allocations: Vec::new(),
                symbol_texts: Default::default(),
                revision_symbol_references: Vec::new(),
                patch,
                descriptive_touches,
                diagnostics_summary,
                lineage,
                derived_index_artifacts,
                schema_transition,
                schema_continuation_descriptor,
                schema_reconciliation_descriptor,
                descriptor_semantics_version,
            },
        }
    }

    /// Move the unsealed data into an existing owner-local inspection slot.
    /// This allocates nothing and supplies no semantic/publication admission.
    #[cfg(test)]
    pub(crate) fn into_body(self) -> CanonicalCommitEnvelopeBody {
        self.body
    }

    pub(crate) fn seal(
        self,
        semantic_result: std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult>,
    ) -> Result<
        CanonicalCommitEnvelope,
        crate::transactions::data::RelationalCommitSemanticResultRefusal,
    > {
        let envelope = CanonicalCommitEnvelope {
            original_inputs: Default::default(),
            live_managed_source_history: Default::default(),
            decoded_bytes: None,
            decoded_memory: None,
            control_allocation: None,
            body: self.body,
            semantic_result,
        };
        envelope.semantic_result().validate_envelope(&envelope)?;
        Ok(envelope)
    }
}
