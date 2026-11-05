//! Current native canonical envelope input: managed text remains borrowed until admission.
#[path = "text_wire/cascade_memory.rs"]
mod cascade_memory;
#[path = "text_wire/seed.rs"]
mod seed;
use super::{CanonicalCommitAuthorityKind, RelationalDescriptiveTouchGraph};
pub(crate) use seed::CanonicalEnvelopeSeed;
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
pub(crate) struct CanonicalEnvelopeWire<Symbols> {
    index_definition_delta: super::CanonicalIndexDefinitionDelta,
    semantic_result: std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult>,
    caller_association: Option<super::super::RelationalCallerAssociation>,
    content: crate::content::ContentPublication,
    required_output_codec_retirements:
        Vec<std::sync::Arc<super::CanonicalRequiredOutputCodecRetirement>>,
    #[serde(default)]
    required_output_origin: Option<std::sync::Arc<super::CanonicalRequiredOutputOrigin>>,
    #[serde(default)]
    producer_cascade_retirement: Option<std::sync::Arc<super::CanonicalProducerRetirement>>,
    authored_cascade_retirement: Option<std::sync::Arc<super::CanonicalAuthoredRetirement>>,
    authored_cascade_restoration: Option<std::sync::Arc<super::CanonicalAuthoredRestoration>>,
    #[serde(default)]
    required_output_materialization:
        Option<std::sync::Arc<super::CanonicalRequiredOutputMaterialization>>,
    #[serde(default)]
    authored_index_writes: super::CanonicalAuthoredIndexWrites,
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
    symbol_texts: Symbols,
    revision_symbol_references: Vec<crate::symbols::data::Symbol>,
    #[serde(default)]
    record_allocations: Vec<crate::history::data::CanonicalRecordAllocation>,
    patch: CanonicalAuthoritativePatch,
    #[serde(default = "RelationalDescriptiveTouchGraph::unavailable")]
    descriptive_touches: RelationalDescriptiveTouchGraph,
    diagnostics_summary: RelationalDiagnosticArtifact,
    lineage: PublishedLineageArtifact,
    derived_index_artifacts: DerivedIndexArtifacts,
    schema_transition: Option<SchemaTransitionArtifact>,
    schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
    schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    // LAST: on every decode/readmission refusal, actual fields drop before
    // their original admissions. This is not a serialized semantic field.
    #[serde(skip)]
    readmission_memory: super::CanonicalEnvelopeReadmissionMemory,
}
impl<Symbols> CanonicalEnvelopeWire<Symbols> {
    pub(crate) fn retain_readmission_memory(
        &mut self,
        memory: super::CanonicalEnvelopeReadmissionMemory,
    ) {
        self.readmission_memory = memory;
    }

    pub(crate) fn attach_shared_readmission_memory(&mut self) -> Result<(), &'static str> {
        self.attach_cascade_memory()?;
        let memory = self.readmission_memory.index_delta.take();
        if let Err(memory) = self.index_definition_delta.retain_decoded_memory(memory) {
            self.readmission_memory.index_delta = memory;
            return Err("decoded index delta backing aliased or already funded before handoff");
        }
        let memory = self.readmission_memory.index_writes.take();
        if let Err(memory) = self.authored_index_writes.retain_decoded_memory(memory) {
            self.readmission_memory.index_writes = memory;
            return Err("decoded authored-index backing aliased or already funded before handoff");
        }
        if let Some(memory) = self.readmission_memory.codec_declarations.as_mut() {
            memory.attach(&self.required_output_codec_retirements)?;
        }
        if let Some(memory) = self.readmission_memory.validation.as_mut() {
            memory.attach(self.semantic_result.validation())?;
        }
        if let Some(memory) = self.readmission_memory.schema.as_mut() {
            memory.attach(self.schema_transition.as_ref())?;
        }
        match (
            self.readmission_memory.strategy.as_mut(),
            self.strategy_artifacts.as_mut(),
        ) {
            (None, None) => {}
            (Some(memory), Some(strategy)) => memory.attach(strategy)?,
            _ => return Err("canonical strategy admission does not match its decoded subject"),
        }
        match (
            self.readmission_memory.merge.as_mut(),
            self.merge_execution_authority.as_ref(),
        ) {
            (None, None) => Ok(()),
            (Some(memory), Some(actual)) => memory.attach(actual),
            _ => Err("canonical merge admission does not match its decoded subject"),
        }
    }

    /// Commit identity as decoded, before any admission. An enclosing frame
    /// compares it so disagreement refuses before text readmission runs.
    pub(crate) fn commit_id(&self) -> CommitId {
        self.commit.commit_id
    }
}
impl CanonicalEnvelopeWire<crate::symbols::data::SymbolSnapshotWire<'_>> {
    fn validate_symbol_coverage(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<worth_execution::ExecutionMemoryReservation>,
        crate::symbols::data::RelationalFieldTextAdmissionDenial,
    > {
        use crate::symbols::data::{
            visit_required_commit_symbols, RelationalFieldTextAdmissionDenial as Denial, Symbol,
        };
        if self
            .revision_symbol_references
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(Denial::InvalidSymbolDictionary);
        }
        let mut count = Some(self.revision_symbol_references.len());
        visit_required_commit_symbols(&self.merged_plan, &self.patch, &mut |_| {
            count = count.and_then(|count| count.checked_add(1));
        });
        let count = count.ok_or(Denial::Request(
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
        ))?;
        let bytes = count
            .checked_add(self.symbol_texts.entries.len())
            .and_then(|count| count.checked_mul(size_of::<Symbol>()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            ))?;
        let funding = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        // Ticket precedes both exact-capacity rosters and remains live through
        // managed-text admission. No private std tree-node estimate is used.
        let mut required = Vec::with_capacity(count);
        visit_required_commit_symbols(&self.merged_plan, &self.patch, &mut |symbol| {
            required.push(symbol)
        });
        required.extend(self.revision_symbol_references.iter().copied());
        required.sort_unstable();
        required.dedup();
        let mut supplied = Vec::with_capacity(self.symbol_texts.entries.len());
        supplied.extend(self.symbol_texts.entries.iter().map(|(symbol, _)| *symbol));
        supplied.sort_unstable();
        if required != supplied {
            return Err(Denial::InvalidSymbolDictionary);
        }
        Ok(funding)
    }

    pub(crate) fn readmit(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        super::CanonicalCommitEnvelope,
        crate::symbols::data::RelationalFieldTextAdmissionDenial,
    > {
        // Every Arc here was created by this single private decode and has
        // not escaped or acquired Weak observers. Attach its already-admitted
        // allocation BEFORE any text/readmission operation can refuse.
        std::sync::Arc::get_mut(&mut self.semantic_result)
            .ok_or(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary,
            )?
            .decoded_memory = self.readmission_memory.semantic.take();
        if let Some(association) = self.caller_association.take() {
            self.caller_association =
                Some(association.with_decoded_memory(self.readmission_memory.association.take()));
        }
        if let Some(origin) = self.required_output_origin.as_mut() {
            std::sync::Arc::get_mut(origin)
                .ok_or(crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary)?
                .retain_decoded_memory(self.readmission_memory.origin.take());
        }
        let _symbol_scratch = self.validate_symbol_coverage(execution)?;
        let symbol_texts = std::mem::replace(
            &mut self.symbol_texts,
            crate::symbols::data::SymbolSnapshotWire {
                entries: Vec::new(),
            },
        )
        .readmit(execution)?;
        self.into_envelope(symbol_texts).map_err(|_| {
            crate::symbols::data::RelationalFieldTextAdmissionDenial::InvalidSymbolDictionary
        })
    }
}
impl<Symbols> CanonicalEnvelopeWire<Symbols> {
    pub(super) fn into_envelope(
        mut self,
        symbol_texts: crate::symbols::data::SymbolTableSnapshot,
    ) -> Result<super::CanonicalCommitEnvelope, &'static str> {
        let body = super::CanonicalCommitEnvelopeBody {
            content: self.content,
            index_definition_delta: self.index_definition_delta,
            caller_association: self.caller_association,
            required_output_codec_retirements: self.required_output_codec_retirements,
            required_output_origin: self.required_output_origin,
            producer_cascade_retirement: self.producer_cascade_retirement,
            authored_cascade_retirement: self.authored_cascade_retirement,
            authored_cascade_restoration: self.authored_cascade_restoration,
            required_output_materialization: self.required_output_materialization,
            authored_index_writes: self.authored_index_writes,
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
            revision_symbol_references: self.revision_symbol_references,
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
        };
        let envelope = super::CanonicalCommitEnvelope {
            original_inputs: Default::default(),
            live_managed_source_history: Default::default(),
            decoded_bytes: self.readmission_memory.input.take(),
            decoded_memory: self.readmission_memory.envelope.take(),
            control_allocation: self.readmission_memory.control.take(),
            body,
            semantic_result: self.semantic_result,
        };
        if !envelope.codec_retirement_headers_valid() {
            return Err("invalid canonical codec retirement headers");
        }
        Ok(envelope)
    }
}

impl CanonicalEnvelopeWire<crate::symbols::data::SymbolTableSnapshot> {
    pub(super) fn take_descriptive_symbols(&mut self) -> crate::symbols::data::SymbolTableSnapshot {
        std::mem::take(&mut self.symbol_texts)
    }
}
