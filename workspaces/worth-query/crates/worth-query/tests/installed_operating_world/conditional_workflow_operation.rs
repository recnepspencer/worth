use std::sync::{Arc, Mutex};

use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation};

use super::conditional_node_contract::{
    conditional_node_result, dependency, node, GeometryCondition,
};
use super::installed_operation_fixture::{
    conditional_installation, operation_conditional_workflow_workspace_with, GeometryDomain,
    ReadFamily, WorkflowRead,
};

#[test]
fn eligible_operation_condition_enters_the_run_before_any_stage_work() {
    let node = node(
        "workflow-operation-eligible",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&node);
    let captured = Arc::new(Mutex::new(None));
    let mut workspace = operation_conditional_workflow_workspace_with(
        "workflow-operation-eligible",
        node,
        installation,
        CapturingWorkflowCompute(Arc::clone(&captured)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let admitted = bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    assert_eq!(admitted.resources().counters().runtime_authority_checks, 1);
    let run = admitted.start_workflow(&mut workspace).unwrap();

    let context = captured.lock().unwrap().take().unwrap();
    assert_eq!(context.run_identity.as_deref(), Some(run.identity()));
    assert!(!context.snapshot_identity.is_empty());
    assert_eq!(context.attempt, 1);
    assert_eq!(run.operation_conditional_provenance().len(), 1);
    assert_eq!(
        run.operation_conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(run.counters().runtime_authority_checks, 0);
    assert_eq!(run.counters().conditional_compute_contacts, 1);
    assert_eq!(run.counters().stage_admission_checks, 0);
    assert_eq!(run.counters().stage_executor_contacts, 0);

    let trace = complete_standard_workflow(run, &mut workspace);
    assert_eq!(trace.operation_conditional_provenance().len(), 1);
    assert_eq!(trace.semantics().operation_conditional_path().len(), 1);
}

#[test]
fn deferred_operation_condition_returns_fresh_retry_authority_and_zero_stage_work() {
    let node = domain_condition_node("workflow-operation-deferred");
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Deferred,
        ));
    let compute_contacts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut workspace = operation_conditional_workflow_workspace_with(
        "workflow-operation-deferred",
        node,
        installation,
        CountedWorkflowCompute(Arc::clone(&compute_contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Deferred(first) = bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
    else {
        panic!("the operation-level condition must defer workflow creation")
    };
    assert_zero_stage_work(first.counters());
    assert_eq!(first.attempt(), 1);
    assert_eq!(
        first.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredByCondition
    );
    let first_identity = first.run_identity().to_owned();

    let TransitionOutcome::Deferred(second) = first.retry(&mut workspace) else {
        panic!("a retry must perform a fresh operation-level conditional attempt")
    };
    assert_zero_stage_work(second.counters());
    assert_eq!(second.attempt(), 2);
    assert_ne!(second.run_identity(), first_identity);
    assert_eq!(
        compute_contacts.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[test]
fn ineligible_operation_condition_cannot_create_a_workflow_run() {
    let node = domain_condition_node("workflow-operation-suppressed");
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Suppressed,
        ));
    let compute_contacts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut workspace = operation_conditional_workflow_workspace_with(
        "workflow-operation-suppressed",
        node,
        installation,
        CountedWorkflowCompute(Arc::clone(&compute_contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Deferred(stopped) = bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
    else {
        panic!("a suppressed operation-level condition must not mint a workflow run")
    };

    assert_zero_stage_work(stopped.counters());
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::Suppressed
    );
    assert_eq!(
        compute_contacts.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

fn complete_standard_workflow(
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

fn assert_zero_stage_work(counters: domain::WorthQueryWorkflowRunCounters) {
    assert_eq!(counters.stage_index_lookups, 0);
    assert_eq!(counters.stage_admission_checks, 0);
    assert_eq!(counters.predecessor_checks, 0);
    assert_eq!(counters.graph_read_contacts, 0);
    assert_eq!(counters.touch_effect_contacts, 0);
    assert_eq!(counters.stage_executor_contacts, 0);
}

fn domain_condition_node(identity: &str) -> domain::WorthQueryPortableConditionalNodeDeclaration {
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

struct CapturedWorkflowContext {
    run_identity: Option<String>,
    snapshot_identity: String,
    attempt: u64,
}

struct CapturingWorkflowCompute(Arc<Mutex<Option<CapturedWorkflowContext>>>);

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
        crate::suite::installed_operation_fixture::execution_resource_support()
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

struct StaticCondition(worth_signal::facade::InstalledSignalConditionDecision);

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

struct CountedWorkflowCompute(Arc<std::sync::atomic::AtomicUsize>);

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
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        changed_result()
    }
}

fn changed_result() -> Result<worth_signal::facade::NodeEvaluationResult, String> {
    Ok(worth_signal::facade::NodeEvaluationResult::from_version(
        worth_signal::facade::AspectVersion::from_updates([(
            worth_signal::facade::Aspect::new(0),
            1,
        )]),
    ))
}
