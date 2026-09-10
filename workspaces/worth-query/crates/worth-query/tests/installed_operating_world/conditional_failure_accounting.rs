use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::conditional_node_contract::node;
use super::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

#[test]
fn failed_conditional_compute_retains_exact_lower_runtime_work() {
    let declaration = node(
        "failed-compute-accounting",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&declaration);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "failed-compute-accounting",
        declaration,
        installation,
        FailingCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();

    let TransitionOutcome::Failed(denial) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("the installed conditional compute failure must remain a checked failure")
    };

    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        denial.kind(),
        domain::WorthQueryBoundExecutionDenialKind::ConditionalExecution(
            worth_runtime_bridge::facade::BridgeConditionalDenialKind::SignalExecution
        )
    ));
    assert_eq!(denial.counters().conditional_dependency_checks, 1);
    assert_eq!(denial.counters().conditional_condition_checks, 1);
    assert_eq!(denial.counters().conditional_compute_contacts, 1);
    assert_eq!(denial.counters().conditional_semantic_changes, 0);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().executor_contacts, 0);
}

struct FailingCompute(Arc<AtomicUsize>);

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
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("declared compute failed".into())
    }
}
