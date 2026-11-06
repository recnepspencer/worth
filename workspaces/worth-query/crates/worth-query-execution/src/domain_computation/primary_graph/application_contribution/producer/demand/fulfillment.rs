//! Lend only the real settled required demand's private interest to its owner.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryFulfilledRequiredRoot,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryOutputDemandSettlement, WorthQueryPreparedRequiredOutputSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};
impl<Schema: ApplicationSchema, Family: WorthQueryProducerOutputFamily<Schema>>
    WorthQueryAdmittedOutputDemand<Schema, Family>
{
    pub(in crate::domain_computation::primary_graph) fn retain_fulfilled_root(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryApplicationCommitReceipt,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial> {
        if self.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || self.schema_binding != runtime.installed_schema.binding_identity()
            || !settlement.belongs_to(runtime) || !self.settled
            || matches!(self.admission_kind, crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Ordinary) {
            return Err(WorthQueryOutputDemandDenial::new(Kind::ForeignDemand, ""));
        }
        let interest = self
            .interest
            .as_ref()
            .ok_or_else(|| WorthQueryOutputDemandDenial::new(Kind::Closed, ""))?;
        runtime
            .runtime
            .output_demands
            .retain_fulfilled_root(execution, prepared, receipt, interest, settlement)
    }
}
