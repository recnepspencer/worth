pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::Arc;

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use super::conditional_node_contract::node;
pub use super::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

pub struct FailingCompute(pub Arc<AtomicUsize>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for FailingCompute
{
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("declared compute failed".into())
    }
}
