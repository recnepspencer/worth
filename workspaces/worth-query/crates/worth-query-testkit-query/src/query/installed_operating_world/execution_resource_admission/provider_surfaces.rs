pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::Arc;

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, installed};

pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::conditional_node_contract::node;
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, execution_resource_request,
    workflow_workspace_with_parallel_provider, GeometryDomain, ReadExecutionInput, ReadFamily,
    ReadVertex, WorkflowRead,
};

pub fn mismatched_fixture_support() -> domain::WorthQueryExecutionResourceSupport {
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
        domain::WorthQueryExecutionResourceEnvelope::bounded(
            1_000_000,
            1_000_000,
            domain::WorthQueryExecutionMode::Synchronous,
            domain::WorthQueryCancellationSafePointFamily::new("incompatible-safe-point").unwrap(),
        ),
        std::sync::Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint("mismatched-provider-surface", 8)
                .unwrap(),
        ),
    )
}

pub struct ResourceConditionalProvider {
    pub support: domain::WorthQueryExecutionResourceSupport,
    pub contacts: Arc<AtomicUsize>,
}

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for ResourceConditionalProvider
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
        self.support.clone()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        Err("resource admission must prevent this contact".into())
    }
}

pub struct ResourceParallelProvider {
    pub support: domain::WorthQueryExecutionResourceSupport,
    pub contacts: Arc<AtomicUsize>,
}

impl domain::WorthQueryWorkflowParallelAdmissionProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for ResourceParallelProvider
{
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.support.clone()
    }

    fn admit_parallel_frontier(
        &self,
        _call: &domain::WorthQueryWorkflowParallelAdmissionCall,
    ) -> Result<
        worth_signal::facade::adapters::FrontierRouteEvidenceReceipt,
        domain::WorthQueryWorkflowParallelAdmissionFailure,
    > {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        Err(domain::WorthQueryWorkflowParallelAdmissionFailure::new(
            "resource admission must prevent this contact",
        ))
    }
}
