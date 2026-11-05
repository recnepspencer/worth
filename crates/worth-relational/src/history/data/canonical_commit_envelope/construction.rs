//! Construct one complete authoritative envelope; no field has an implicit byte closure.
use super::*;

impl CanonicalCommitEnvelope {
    pub(crate) fn complete_lineage(
        &mut self,
        lineage: crate::lineage::data::LineageFinalizationArtifact,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        if lineage.branch_id() != &self.branch_context || !self.lineage_events().is_empty() {
            return Err(crate::mvcc::origin_denial(
                "deferred publication lineage differs from its draft",
            ));
        }
        self.lineage = lineage.publish();
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        content: crate::content::ContentPublication,
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
            content,
            authored_index_writes: CanonicalAuthoredIndexWrites::default(),
            live_managed_source_history: LiveManagedSourceHistory::default(),
            required_output_codec_retirements: Vec::new(),
            required_output_origin: None,
            historical_output_selection: None,
            producer_cascade_retirement: None,
            authored_cascade_retirement: None,
            authored_cascade_restoration: None,
            coupled_authored_publication: None,
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
        }
    }
}
