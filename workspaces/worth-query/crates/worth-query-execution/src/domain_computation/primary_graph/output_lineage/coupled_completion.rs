//! Hot completion custody for an already performed coupled producer.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::primary_graph::{
    application_query::AppliedSourcePublication, WorthQueryApplicationCommitReceipt,
};
#[derive(Clone)]
pub struct CoupledOutputCompletion {
    source: AppliedSourcePublication,
    receipt: Arc<worth_execution::ExecutionArray<OnceLock<WorthQueryApplicationCommitReceipt>>>,
}
impl CoupledOutputCompletion {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: AppliedSourcePublication,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let map = |cause| {
            match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::execution_denial::advancement_scope_stop(cause)), "coupled completion custody"),
            worth_execution::ExecutionRequestArrayDenial::Allocation(_) => WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded, "coupled completion custody"),
        }
        };
        let mut receipt =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, 1).map_err(map)?;
        receipt.push(OnceLock::new()).map_err(map)?;
        Ok(Self {
            source,
            receipt: Arc::new(receipt.seal().map_err(map)?),
        })
    }
    pub(super) fn completed_receipt(
        &self,
        settlement: &Arc<RecordedSettlementIdentity>,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        let receipt = self.receipt[0].get()?;
        receipt
            .exact_output_settlement()
            .filter(|identity| Arc::ptr_eq(identity, settlement))?;
        Some(receipt.clone())
    }
    fn receipt_for_source(
        &self,
        identity: Option<RecordedSourceIdentity>,
        runtime: application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: application_query::WorthQueryCheckpointSourceIdentity,
        settlement: &Arc<RecordedSettlementIdentity>,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        if !identity?.matches_current(runtime, checkpoint) {
            return None;
        }
        self.completed_receipt(settlement)
    }
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &AppliedSourcePublication {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn complete(
        &self,
        receipt: WorthQueryApplicationCommitReceipt,
    ) {
        assert!(
            self.receipt[0].set(receipt).is_ok(),
            "one actual managed completion for the coupled writer"
        );
    }
}
impl input_cutoff::RetainedInputCutoffCandidate {
    pub(in crate::domain_computation::primary_graph) fn completed_coupled_receipt(
        &self,
        runtime: application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: application_query::WorthQueryCheckpointSourceIdentity,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        let recorded = self.recorded();

        recorded.coupled_completion.as_ref()?.receipt_for_source(
            recorded.source_identity,
            runtime,
            checkpoint,
            &recorded.settlement_identity,
        )
    }
}

impl current_output::retained_candidates::WorthQueryRetainedOutputCandidate {
    pub(in crate::domain_computation::primary_graph) fn completed_coupled_receipt(
        &self,
        runtime: application_query::WorthQueryRuntimeSourceIdentity,
        checkpoint: application_query::WorthQueryCheckpointSourceIdentity,
    ) -> Option<WorthQueryApplicationCommitReceipt> {
        self.coupled_completion.as_ref()?.receipt_for_source(
            self.source_identity,
            runtime,
            checkpoint,
            &self.settlement_identity,
        )
    }
}
