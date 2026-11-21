//! Crate-boundary internal port for `WorthQueryAdmittedOutputDemand`.
//! Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandDenial, WorthQueryOutputReadCloseDenial,
    WorthQueryProducerOutputFamily,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandSettlement,
    WorthQueryOutputSourceEvidence, WorthQueryPreparedRequiredOutputSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub trait AdmittedOutputDemandInternalPort<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    fn retain_fulfilled_root(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryOutputSourceEvidence,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial>;

    fn observe_own_completed_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Option<Arc<WorthQueryOutputDemandSettlement>>, WorthQueryOutputDemandDenial>;

    fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputReadCloseDenial>;
}

impl<Schema, Family> AdmittedOutputDemandInternalPort<Schema, Family>
    for WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    fn retain_fulfilled_root(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryOutputSourceEvidence,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial> {
        Self::retain_fulfilled_root(self, runtime, execution, prepared, receipt, settlement)
    }

    fn observe_own_completed_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Option<Arc<WorthQueryOutputDemandSettlement>>, WorthQueryOutputDemandDenial> {
        Self::observe_own_completed_publication(self, execution, runtime)
    }

    fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        Self::close_read_request(self, phase, runtime, branch)
    }
}
