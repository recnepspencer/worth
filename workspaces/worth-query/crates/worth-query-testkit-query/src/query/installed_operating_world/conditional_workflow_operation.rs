pub use std::sync::{Arc, Mutex};

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, foundation};

pub use super::conditional_node_contract::{
    conditional_node_result, dependency, node, GeometryCondition,
};
pub use super::installed_operation_fixture::{
    conditional_installation, operation_conditional_workflow_workspace_with, GeometryDomain,
    ReadFamily, WorkflowRead,
};

pub fn complete_standard_workflow(
    run: domain::WorthQueryWorkflowRun<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
        foundation::ObservationLaneWitness,
    >,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryCompletedWorkflowTrace<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    run.advance(
        "start",
        domain::WorthQueryWorkflowValue::NotRequired,
        workspace,
    )
    .unwrap()
    .advance(
        "left",
        domain::WorthQueryWorkflowValue::Text("start".into()),
        workspace,
    )
    .unwrap()
    .advance(
        "right",
        domain::WorthQueryWorkflowValue::Text("start".into()),
        workspace,
    )
    .unwrap()
    .advance(
        "publish",
        domain::WorthQueryWorkflowValue::Text("join".into()),
        workspace,
    )
    .unwrap()
    .complete()
    .unwrap()
}

pub fn assert_zero_stage_work(counters: domain::WorthQueryWorkflowRunCounters) {
    assert_eq!(counters.stage_index_lookups, 0);
    assert_eq!(counters.stage_admission_checks, 0);
    assert_eq!(counters.predecessor_checks, 0);
    assert_eq!(counters.graph_read_contacts, 0);
    assert_eq!(counters.touch_effect_contacts, 0);
    assert_eq!(counters.stage_executor_contacts, 0);
}

pub fn domain_condition_node(
    identity: &str,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node_result(
        identity,
        dependency(domain::WorthQuerySemanticLocality::SourceRecord),
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap()
}

pub struct CapturedWorkflowContext {
    pub run_identity: Option<String>,
    pub snapshot_identity: String,
    pub attempt: u64,
}

pub struct CapturingWorkflowCompute(pub Arc<Mutex<Option<CapturedWorkflowContext>>>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for CapturingWorkflowCompute
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
        context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        *self.0.lock().unwrap() = Some(CapturedWorkflowContext {
            run_identity: context.workflow_run_identity().map(str::to_owned),
            snapshot_identity: context.snapshot_identity().to_owned(),
            attempt: context.attempt(),
        });
        changed_result()
    }
}

pub struct StaticCondition(pub worth_signal::facade::InstalledSignalConditionDecision);

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for StaticCondition {
    type SemanticContract = worth_signal::facade::InstalledSignalConditionDecision;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalConditionProvider for StaticCondition {
    fn resolve(
        &self,
        _: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(self.0)
    }
}

pub struct CountedWorkflowCompute(pub Arc<std::sync::atomic::AtomicUsize>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for CountedWorkflowCompute
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
        _: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        changed_result()
    }
}

pub fn changed_result() -> Result<worth_signal::facade::NodeEvaluationResult, String> {
    Ok(worth_signal::facade::NodeEvaluationResult::from_version(
        worth_signal::facade::AspectVersion::from_updates([(
            worth_signal::facade::Aspect::new(0),
            1,
        )]),
    ))
}

pub mod advancement_custody;
