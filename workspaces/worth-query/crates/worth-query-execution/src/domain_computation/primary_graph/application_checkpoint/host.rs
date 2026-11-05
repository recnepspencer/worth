//! What `application_checkpoint` needs from the runtime that composes it.

/// The runtime's identity plus what `application_checkpoint` reads or calls above itself.
pub trait ApplicationCheckpointHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `application_checkpoint` is handed.
    fn application_checkpoint_host(&self) -> &dyn ApplicationCheckpointHost<Schema>;
    fn publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::bootstrap::WorthQueryPrimaryGraphPublication;
    fn output_demands(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;
    fn recovered_outputs(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs;
    fn application_branch_commit_lane_for_occurrence(&self, occurrence: worth_runtime_world::facade::ProductBranchIncarnation) -> std::sync::Arc<crate::domain_computation::primary_graph::provider::branch_commit_coordination::WorthQueryApplicationBranchCommitLane>;
}
