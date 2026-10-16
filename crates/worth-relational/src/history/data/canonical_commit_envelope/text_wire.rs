//! Current native canonical envelope input: managed text remains borrowed until admission.
use super::{CanonicalCommitAuthorityKind, RelationalDescriptiveTouchGraph};
use serde::Deserialize;

use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::PublishedLineageArtifact;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaAuthoritySnapshot, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor, SchemaTransitionArtifact, SchemaVersionId,
};
use crate::transactions::data::{MergedCommitPlan, PublishedMergeExecutionAuthority};

#[derive(Deserialize)]
pub(crate) struct CanonicalEnvelopeWire<'a> {
    commit: RelationalCommitReceipt,
    branch_context: BranchId,
    branch_cell_checkpoint: Option<crate::branch::RelationalBranchCellCheckpoint>,
    authority_kind: CanonicalCommitAuthorityKind,
    strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    merge_parent_branches: Vec<BranchId>,
    merge_base_commits: Vec<CommitId>,
    schema_version: SchemaVersionId,
    schema_authority: SchemaAuthoritySnapshot,
    merged_plan: MergedCommitPlan,
    #[serde(borrow)]
    symbol_texts: crate::symbols::data::SymbolSnapshotWire<'a>,
    record_allocations: Vec<crate::history::data::CanonicalRecordAllocation>,
    patch: CanonicalAuthoritativePatch,
    descriptive_touches: RelationalDescriptiveTouchGraph,
    diagnostics_summary: RelationalDiagnosticArtifact,
    lineage: PublishedLineageArtifact,
    derived_index_artifacts: DerivedIndexArtifacts,
    schema_transition: Option<SchemaTransitionArtifact>,
    schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
    schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
}
impl CanonicalEnvelopeWire<'_> {
    pub(crate) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        super::CanonicalCommitEnvelope,
        crate::symbols::data::RelationalFieldTextAdmissionDenial,
    > {
        let required =
            crate::symbols::data::required_commit_symbols(&self.merged_plan, &self.patch);
        let supplied = self
            .symbol_texts
            .entries
            .iter()
            .map(|(symbol, _)| *symbol)
            .collect::<std::collections::BTreeSet<_>>();
        if required != supplied || self.symbol_texts.entries.len() != required.len() {
            return Err(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary,
            );
        }
        let symbol_texts = self.symbol_texts.readmit(execution)?;
        Ok(super::CanonicalCommitEnvelope {
            commit: self.commit,
            branch_context: self.branch_context,
            branch_cell_checkpoint: self.branch_cell_checkpoint,
            authority_kind: self.authority_kind,
            strategy_artifacts: self.strategy_artifacts,
            merge_execution_authority: self.merge_execution_authority,
            merge_parent_branches: self.merge_parent_branches,
            merge_base_commits: self.merge_base_commits,
            schema_version: self.schema_version,
            schema_authority: self.schema_authority,
            merged_plan: self.merged_plan,
            symbol_texts,
            record_allocations: self.record_allocations,
            patch: self.patch,
            descriptive_touches: self.descriptive_touches,
            diagnostics_summary: self.diagnostics_summary,
            lineage: self.lineage,
            derived_index_artifacts: self.derived_index_artifacts,
            schema_transition: self.schema_transition,
            schema_continuation_descriptor: self.schema_continuation_descriptor,
            schema_reconciliation_descriptor: self.schema_reconciliation_descriptor,
            descriptor_semantics_version: self.descriptor_semantics_version,
        })
    }
}
