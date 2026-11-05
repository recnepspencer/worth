//! What the output-demand subsystem needs from the runtime that composes it.

use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::product_operation::{
    WorthQueryProductEntry, WorthQuerySelectedProductOperation,
};

/// The runtime's identity, its graph handle, and the product selections output demands read.
pub trait OutputDemandHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object output-demand state is handed.
    fn output_demand_host(&self) -> &dyn OutputDemandHost<Schema>;
    fn take_ready_read_snapshot_pressure(&self) -> bool;
    fn hold_world_snapshot_pressure_for_test(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Vec<crate::basis::WorthQueryProductBranchLease>;
    fn next_output_producer_attempt(&self) -> &std::sync::atomic::AtomicU64;
    fn on_branch(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> WorthQueryProductEntry<'_, Schema>;
    fn select_application_read_observation_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    >;
}
