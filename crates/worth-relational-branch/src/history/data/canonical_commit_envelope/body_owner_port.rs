//! Owner access to a canonical envelope body.
use super::{
    CanonicalAuthoredIndexWrites, CanonicalAuthoredRestoration, CanonicalAuthoredRetirement,
    CanonicalCommitEnvelopeBody, CanonicalCoupledAuthoredPublication,
    CanonicalHistoricalOutputSelection, CanonicalIndexDefinitionDelta, CanonicalProducerRetirement,
    CanonicalRequiredOutputCodecRetirement, CanonicalRequiredOutputMaterialization,
    CanonicalRequiredOutputOrigin,
};
use crate::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint;
use crate::content::publication::ContentPublication;
use crate::history::data::{
    CancelledRecordAllocation, CanonicalRecordAllocation, RelationalCallerAssociation,
};
use crate::lineage::data::PublishedLineageArtifact;
use crate::symbols::data::{Symbol, SymbolTableSnapshot};
use crate::transactions::data::primitives::RecordRef;
use std::collections::BTreeSet;
use std::sync::Arc;

#[doc(hidden)]
pub trait CanonicalCommitEnvelopeBodyOwnerPort: Sized {
    fn install_record_allocations(&mut self, allocations: Vec<CanonicalRecordAllocation>);
    fn record_allocations(&self) -> &[CanonicalRecordAllocation];
    fn install_cancelled_record_allocations(&mut self, cancelled: Vec<CancelledRecordAllocation>);
    fn cancelled_record_allocations(&self) -> &[CancelledRecordAllocation];
    fn published_lineage(&self) -> &PublishedLineageArtifact;
    fn touched_record_refs(&self) -> BTreeSet<RecordRef>;
    fn authored_intent_targets(&self) -> impl Iterator<Item = Option<RecordRef>> + '_;
}

impl CanonicalCommitEnvelopeBodyOwnerPort for CanonicalCommitEnvelopeBody {
    fn install_record_allocations(&mut self, allocations: Vec<CanonicalRecordAllocation>) {
        self.install_record_allocations(allocations)
    }

    fn record_allocations(&self) -> &[CanonicalRecordAllocation] {
        self.record_allocations()
    }

    fn install_cancelled_record_allocations(&mut self, cancelled: Vec<CancelledRecordAllocation>) {
        self.install_cancelled_record_allocations(cancelled)
    }

    fn cancelled_record_allocations(&self) -> &[CancelledRecordAllocation] {
        self.cancelled_record_allocations()
    }

    fn published_lineage(&self) -> &PublishedLineageArtifact {
        self.published_lineage()
    }

    fn touched_record_refs(&self) -> BTreeSet<RecordRef> {
        self.touched_record_refs()
    }

    fn authored_intent_targets(&self) -> impl Iterator<Item = Option<RecordRef>> + '_ {
        self.authored_intent_targets()
    }
}

#[doc(hidden)]
pub trait CanonicalCommitEnvelopeBodyFieldsPort {
    fn owner_authored_cascade_restoration_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalAuthoredRestoration>>;
    fn owner_authored_cascade_restoration(&self) -> &Option<Arc<CanonicalAuthoredRestoration>>;
    fn owner_authored_cascade_retirement_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalAuthoredRetirement>>;
    fn owner_authored_cascade_retirement(&self) -> &Option<Arc<CanonicalAuthoredRetirement>>;
    fn owner_authored_index_writes_mut(&mut self) -> &mut CanonicalAuthoredIndexWrites;
    fn owner_authored_index_writes(&self) -> &CanonicalAuthoredIndexWrites;
    fn owner_branch_cell_checkpoint_mut(&mut self) -> &mut Option<RelationalBranchCellCheckpoint>;
    fn owner_branch_cell_checkpoint(&self) -> &Option<RelationalBranchCellCheckpoint>;
    fn owner_caller_association_mut(&mut self) -> &mut Option<RelationalCallerAssociation>;
    fn owner_content(&self) -> &ContentPublication;
    fn owner_coupled_authored_publication_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalCoupledAuthoredPublication>>;
    fn owner_coupled_authored_publication(
        &self,
    ) -> &Option<Arc<CanonicalCoupledAuthoredPublication>>;
    fn owner_historical_output_selection(&self)
        -> &Option<Arc<CanonicalHistoricalOutputSelection>>;
    fn owner_index_definition_delta(&self) -> &CanonicalIndexDefinitionDelta;
    fn owner_producer_cascade_retirement_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalProducerRetirement>>;
    fn owner_producer_cascade_retirement(&self) -> &Option<Arc<CanonicalProducerRetirement>>;
    fn owner_required_output_codec_retirements_mut(
        &mut self,
    ) -> &mut Vec<Arc<CanonicalRequiredOutputCodecRetirement>>;
    fn owner_required_output_codec_retirements(
        &self,
    ) -> &Vec<Arc<CanonicalRequiredOutputCodecRetirement>>;
    fn owner_required_output_materialization_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalRequiredOutputMaterialization>>;
    fn owner_required_output_materialization(
        &self,
    ) -> &Option<Arc<CanonicalRequiredOutputMaterialization>>;
    fn owner_required_output_origin_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalRequiredOutputOrigin>>;
    fn owner_required_output_origin(&self) -> &Option<Arc<CanonicalRequiredOutputOrigin>>;
    fn owner_revision_symbol_references_mut(&mut self) -> &mut Vec<Symbol>;
    fn owner_revision_symbol_references(&self) -> &Vec<Symbol>;
    fn owner_symbol_texts_mut(&mut self) -> &mut SymbolTableSnapshot;
    fn owner_symbol_texts(&self) -> &SymbolTableSnapshot;
}

impl CanonicalCommitEnvelopeBodyFieldsPort for CanonicalCommitEnvelopeBody {
    fn owner_authored_cascade_restoration_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalAuthoredRestoration>> {
        &mut self.authored_cascade_restoration
    }
    fn owner_authored_cascade_restoration(&self) -> &Option<Arc<CanonicalAuthoredRestoration>> {
        &self.authored_cascade_restoration
    }
    fn owner_authored_cascade_retirement_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalAuthoredRetirement>> {
        &mut self.authored_cascade_retirement
    }
    fn owner_authored_cascade_retirement(&self) -> &Option<Arc<CanonicalAuthoredRetirement>> {
        &self.authored_cascade_retirement
    }
    fn owner_authored_index_writes_mut(&mut self) -> &mut CanonicalAuthoredIndexWrites {
        &mut self.authored_index_writes
    }
    fn owner_authored_index_writes(&self) -> &CanonicalAuthoredIndexWrites {
        &self.authored_index_writes
    }
    fn owner_branch_cell_checkpoint_mut(&mut self) -> &mut Option<RelationalBranchCellCheckpoint> {
        &mut self.branch_cell_checkpoint
    }
    fn owner_branch_cell_checkpoint(&self) -> &Option<RelationalBranchCellCheckpoint> {
        &self.branch_cell_checkpoint
    }
    fn owner_caller_association_mut(&mut self) -> &mut Option<RelationalCallerAssociation> {
        &mut self.caller_association
    }
    fn owner_content(&self) -> &ContentPublication {
        &self.content
    }
    fn owner_coupled_authored_publication_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalCoupledAuthoredPublication>> {
        &mut self.coupled_authored_publication
    }
    fn owner_coupled_authored_publication(
        &self,
    ) -> &Option<Arc<CanonicalCoupledAuthoredPublication>> {
        &self.coupled_authored_publication
    }
    fn owner_historical_output_selection(
        &self,
    ) -> &Option<Arc<CanonicalHistoricalOutputSelection>> {
        &self.historical_output_selection
    }
    fn owner_index_definition_delta(&self) -> &CanonicalIndexDefinitionDelta {
        &self.index_definition_delta
    }
    fn owner_producer_cascade_retirement_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalProducerRetirement>> {
        &mut self.producer_cascade_retirement
    }
    fn owner_producer_cascade_retirement(&self) -> &Option<Arc<CanonicalProducerRetirement>> {
        &self.producer_cascade_retirement
    }
    fn owner_required_output_codec_retirements_mut(
        &mut self,
    ) -> &mut Vec<Arc<CanonicalRequiredOutputCodecRetirement>> {
        &mut self.required_output_codec_retirements
    }
    fn owner_required_output_codec_retirements(
        &self,
    ) -> &Vec<Arc<CanonicalRequiredOutputCodecRetirement>> {
        &self.required_output_codec_retirements
    }
    fn owner_required_output_materialization_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalRequiredOutputMaterialization>> {
        &mut self.required_output_materialization
    }
    fn owner_required_output_materialization(
        &self,
    ) -> &Option<Arc<CanonicalRequiredOutputMaterialization>> {
        &self.required_output_materialization
    }
    fn owner_required_output_origin_mut(
        &mut self,
    ) -> &mut Option<Arc<CanonicalRequiredOutputOrigin>> {
        &mut self.required_output_origin
    }
    fn owner_required_output_origin(&self) -> &Option<Arc<CanonicalRequiredOutputOrigin>> {
        &self.required_output_origin
    }
    fn owner_revision_symbol_references_mut(&mut self) -> &mut Vec<Symbol> {
        &mut self.revision_symbol_references
    }
    fn owner_revision_symbol_references(&self) -> &Vec<Symbol> {
        &self.revision_symbol_references
    }
    fn owner_symbol_texts_mut(&mut self) -> &mut SymbolTableSnapshot {
        &mut self.symbol_texts
    }
    fn owner_symbol_texts(&self) -> &SymbolTableSnapshot {
        &self.symbol_texts
    }
}
