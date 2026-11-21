//! Whole-tree qualification at the original tree's canonically linked endpoint.
mod collection;
use super::WorthQueryOutputSettlementPosture;
use crate::domain_computation::primary_graph::application_installation::WorthQuerySelectedProgramOwnerDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryOutputDemandDenial,
    WorthQueryProductBranchAdmissionDenial,
};
use crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial;
pub use collection::{qualify_output_collection, OriginalOutputTree};
use std::sync::Arc;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Original read-source provenance and a qualified endpoint connected only by
/// canonical publications retained by the original complete tree. Shared and
/// recovered output receipts remain lawful; this does not assert exclusive work.
/// The genuine root read is retained separately from E and keeps its original
/// producer correspondence. Its seed is not relabeled as an admission at E.
/// Neither this fact nor its descriptions grant current permission or selection.
pub struct WorthQueryCompletedOriginalReadOutputs {
    source: Arc<WorthQueryApplicationReadObservation>,
    root: Arc<WorthQueryApplicationReadObservation>,
    posture: WorthQueryOutputSettlementPosture,
    output: Arc<WorthQueryApplicationReadObservation>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryCompletedOriginalReadOutputs {
    pub fn from_qualified_endpoint(
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        source: Arc<WorthQueryApplicationReadObservation>,
        root: Arc<WorthQueryApplicationReadObservation>,
        posture: WorthQueryOutputSettlementPosture,
        output: Arc<WorthQueryApplicationReadObservation>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            source,
            root,
            posture,
            output,
            _memory: memory,
        }
    }
    pub fn original_source(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.source
    }
    pub fn original_endpoint(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.output
    }
    pub fn source_commit(&self) -> &CompositeCommitIdentity {
        self.source.selected_commit()
    }
    pub fn root_output_commit(&self) -> &CompositeCommitIdentity {
        self.root.selected_commit()
    }
    pub fn output_commit(&self) -> &CompositeCommitIdentity {
        self.output.selected_commit()
    }
    pub fn root_posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }
}

#[derive(Debug)]
pub enum WorthQueryOriginalReadOutputDenial {
    Output(WorthQueryOutputDemandDenial),
    Observation(WorthQueryProductBranchAdmissionDenial),
    Program(WorthQuerySelectedProgramOwnerDenial),
    Currentness(WorthQueryRequiredSetCurrentnessDenial),
}
