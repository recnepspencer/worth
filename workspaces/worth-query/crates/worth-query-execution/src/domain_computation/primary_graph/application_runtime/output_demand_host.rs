//! The runtime composes the output-demand subsystem: identity, graph handle and selections.
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_output_demand::host::OutputDemandHost;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::product_operation::{
    WorthQueryProductEntry, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

impl<Schema: ApplicationSchema> OutputDemandHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn output_demand_host(&self) -> &dyn OutputDemandHost<Schema> {
        self
    }
    fn take_ready_read_snapshot_pressure(&self) -> bool {
        self.runtime
            .primary_provider
            .take_ready_read_snapshot_pressure()
    }
    fn hold_world_snapshot_pressure_for_test(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Vec<crate::basis::WorthQueryProductBranchLease> {
        Self::hold_world_snapshot_pressure_for_test(self, branch)
    }
    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.installed_schema
    }
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime.authority_identity()
    }
    fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.runtime.product_runtime
    }
    fn provider_graph(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.runtime.primary_provider.graph
    }
    fn next_output_producer_attempt(&self) -> &std::sync::atomic::AtomicU64 {
        &self.runtime.next_output_producer_attempt
    }
    fn on_branch(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> WorthQueryProductEntry<'_, Schema> {
        Self::on_branch(self, branch)
    }
    fn select_application_read_observation_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_application_read_observation_with_request(self, execution, observation)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn select_output_demand_settlement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        settlement: &crate::domain_computation::primary_graph::application_output_demand::settlement::WorthQueryOutputDemandSettlement,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    >{
        <Self as crate::domain_computation::primary_graph::application_output_demand::settlement::SettlementOps<Schema>>::select_output_demand_settlement(self, execution, settlement)
    }
}
