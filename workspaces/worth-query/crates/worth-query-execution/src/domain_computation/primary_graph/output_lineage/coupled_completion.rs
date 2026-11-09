//! Hot completion custody for an already performed coupled producer.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationLandedCommit,
    application_query::AppliedSourcePublication, WorthQueryApplicationCommitReceipt,
};
use crate::domain_computation::primary_graph::{
    application_query, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
#[derive(Clone)]
pub struct CoupledOutputCompletion {
    source: AppliedSourcePublication,
    receipt: Arc<worth_execution::ExecutionArray<OnceLock<WorthQueryApplicationLandedCommit>>>,
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
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &AppliedSourcePublication {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn complete(
        &self,
        receipt: WorthQueryApplicationLandedCommit,
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

        if !recorded
            .source_identity?
            .matches_current(runtime, checkpoint)
        {
            return None;
        }
        let receipt = match recorded.coupled_completion.as_ref()?.receipt[0].get()? {
            WorthQueryApplicationLandedCommit::Live { receipt } => receipt,
            WorthQueryApplicationLandedCommit::Recovered(_) => return None,
        };
        if !receipt
            .exact_output_settlement()
            .is_some_and(|identity| Arc::ptr_eq(identity, &recorded.settlement_identity))
        {
            return None;
        }
        Some(receipt.clone())
    }
}
