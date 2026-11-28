//! Pure read-only seed for an original settled producer observation.

mod court_doors_for_test;
pub mod provenance;

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    output_lineage::RecordedSettlementIdentity,
    source_disclosure::OriginalSourceInspectionDisclosure,
    WorthQueryApplicationCommittedChanges, WorthQueryApplicationOutputCorrespondence,
};
use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

#[derive(Clone)]
pub enum OriginalSettledProducerSource {
    Committed {
        settlement: Option<Arc<RecordedSettlementIdentity>>,
        publication: Option<WorthQueryProductBranchReadIdentity>,
        changes: WorthQueryApplicationCommittedChanges,
        inputs: Option<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >,
        conditional: Option<worth_runtime_bridge::facade::BridgeConditionalCausalObservation>,
        evaluation: Option<
            Arc<crate::domain_computation::primary_graph::application_contribution::ProducerOutputEvaluation>,
        >,
        disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
        original_publication: Option<Arc<super::OriginalProducingPublication>>,
    },
    StableReused {
        settlement: Arc<RecordedSettlementIdentity>,
    },
    Restored {
        checkpoint: super::super::WorthQueryAcceptedOutputCheckpointIdentity,
    },
}

pub struct OriginalSettledProducerObservation {
    pub runtime_authority: u64,
    pub schema: ApplicationSchemaBindingIdentity,
    pub read: WorthQueryProductBranchReadIdentity,
    pub source: OriginalSettledProducerSource,
    pub correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub historical_reads: Option<
        Arc<[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead]>,
    >,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl std::fmt::Debug for OriginalSettledProducerObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalSettledProducerObservation")
            .finish_non_exhaustive()
    }
}

impl OriginalSettledProducerObservation {
    pub fn matches_occurrence(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime_authority
            && &self.schema == schema
            && self.read.branch_identity() == observation.branch_identity()
            && self.read.lifecycle_incarnation() == observation.lifecycle_incarnation()
            && self.read.selected_commit() == observation.selected_commit()
    }

    pub fn source(&self) -> &OriginalSettledProducerSource {
        &self.source
    }

    pub fn correspondence(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }

    pub fn retain_correspondence(&self) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        Arc::clone(&self.correspondence)
    }

    pub fn read(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.read
    }

    pub fn historical_reads(
        &self,
    ) -> Option<
        &[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead],
    >{
        self.historical_reads.as_deref()
    }

    pub fn predecessor_reads(
        &self,
    ) -> Option<
        &[crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead],
    >{
        self.historical_reads.as_deref()
    }
}
