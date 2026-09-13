use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, installed};

use crate::suite::conditional_node_contract::node;
use crate::suite::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, execution_resource_request,
    workflow_workspace_with_parallel_provider, GeometryDomain, ReadExecutionInput, ReadFamily,
    ReadVertex, WorkflowRead,
};

#[test]
fn conditional_provider_mismatch_denies_before_provider_contact() {
    let declaration = node(
        "resource-conditional-provider",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&declaration);
    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = conditional_workspace_with(
        "resource-conditional-provider-mismatch",
        declaration,
        installation,
        ResourceConditionalProvider {
            support: mismatched_fixture_support(),
            contacts: Arc::clone(&contacts),
        },
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        ReadExecutionInput::default(),
        execution_resource_request(),
        &workspace,
    ) else {
        panic!("conditional-provider support mismatch must deny")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains("conditional node"));
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn parallel_provider_mismatch_denies_before_provider_contact() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = workflow_workspace_with_parallel_provider(
        "resource-parallel-provider-mismatch",
        ResourceParallelProvider {
            support: mismatched_fixture_support(),
            contacts: Arc::clone(&contacts),
        },
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) =
        bound.admit_workflow_resources(execution_resource_request(), &workspace)
    else {
        panic!("parallel-provider support mismatch must deny")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains("parallel admission provider"));
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

fn mismatched_fixture_support() -> domain::WorthQueryExecutionResourceSupport {
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

struct ResourceConditionalProvider {
    support: domain::WorthQueryExecutionResourceSupport,
    contacts: Arc<AtomicUsize>,
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

struct ResourceParallelProvider {
    support: domain::WorthQueryExecutionResourceSupport,
    contacts: Arc<AtomicUsize>,
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
