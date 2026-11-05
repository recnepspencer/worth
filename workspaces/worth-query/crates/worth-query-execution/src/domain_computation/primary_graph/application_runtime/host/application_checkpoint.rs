//! The runtime composes `application_checkpoint`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_checkpoint::host::ApplicationCheckpointHost;

impl<Schema: ApplicationSchema> ApplicationCheckpointHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_checkpoint_host(&self) -> &dyn ApplicationCheckpointHost<Schema> {
        self
    }
    fn publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::bootstrap::WorthQueryPrimaryGraphPublication
    {
        self.runtime.publication()
    }
    fn output_demands(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry{
        &self.output_production.demands()
    }
    fn recovered_outputs(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs{
        &self.output_production.recovered()
    }
    fn application_branch_commit_lane_for_occurrence(&self, occurrence: worth_runtime_world::facade::ProductBranchIncarnation) -> std::sync::Arc<crate::domain_computation::primary_graph::provider::branch_commit_coordination::WorthQueryApplicationBranchCommitLane>{
        self.runtime
            .primary_provider
            .application_branch_commit_lane_for_occurrence(occurrence)
    }
}
