use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDirectGraphStepOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowGraphStepOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowRunCleanupOutcome;
use crate::domain_computation::provider_session::tests::admitted_plan;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::direct_yield_cleanup_fixture::complete_direct_yield_cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority;
use crate::domain_computation::managed_run::tests::query_runtime;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::workflow_authority_with_output_artifact;

use crate::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryReadmissionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionOutcome;
pub const UNRELATED_RUN_COUNT: usize = 12;
pub fn unrelated_artifact_run(
    index: usize,
    disposed: Arc<AtomicUsize>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
) {
    let runtime = query_runtime();
    unrelated_artifact_run_in_runtime(&runtime, index, disposed, resource_request)
}
pub fn unrelated_artifact_run_in_runtime(
    runtime: &WorthQueryExecutionRuntime,
    index: usize,
    disposed: Arc<AtomicUsize>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
) {
    let operation_label = format!("unrelated-workflow-{index}");
    let stage_label = format!("unrelated-workflow-{index}:producer");
    let operation_resources = admitted_plan(&operation_label, 8);
    let stage_resources = admitted_plan(&stage_label, 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("producer".to_owned(), stage_resources)]),
    );
    let output =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run();
    let operation =
        workflow_authority_with_output_artifact(runtime, &resources, "producer", output);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("unrelated workflow should reserve");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("unrelated workflow should admit")
        .start()
        .expect("unrelated workflow should start");
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("producer stage should validate")
        .expect("producer should own artifact authority");
    let admission =
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                format!("unrelated-provenance-{index}"),
                format!("unrelated-dependency-{index}"),
            ),
        );
    let handle =
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            CostBoundArtifactResource(disposed),
        )
        .expect("unrelated artifact should register");
    (running, handle)
}
pub fn direct_readmission_work(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime) =
        crate::domain_computation::managed_run::tests::readmission_direct::yielded_direct(
            execution,
            resource_request,
        );
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width, resource_request);
    let readmitted = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
        };
        match fixture_phase_result {
            WorthQueryDirectReadmissionOutcome::Readmitted(readmitted) => readmitted,
            _ => panic!("direct cost target should readmit"),
        }
    };
    let evidence = readmitted.readmission_evidence();
    let terminal = match readmitted.into_active().abandon() {
        WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("direct cost target should explicitly terminalize"),
    };
    terminal
        .cleanup()
        .expect("direct cost target should clean up");
    assert_and_release_unrelated_authority(unrelated, &disposed, unrelated_width);
    evidence
}
pub fn workflow_readmission_work(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime, old_producer) =
        crate::domain_computation::managed_run::tests::readmission_workflow::yielded_workflow(
            execution,
            crate::domain_computation::managed_run::tests::yield_fixture::YieldProvider::installed(
                7,
            ),
            resource_request,
        );
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width, resource_request);
    let readmitted = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
        };
        match fixture_phase_result {
            WorthQueryWorkflowReadmissionOutcome::Readmitted(readmitted) => readmitted,
            _ => panic!("workflow cost target should readmit"),
        }
    };
    let evidence = readmitted.readmission_evidence();
    drop(old_producer);
    let terminal = match readmitted.into_active().abandon() {
        WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("workflow cost target should explicitly terminalize"),
    };
    match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
        _ => panic!("workflow cost target should clean up"),
    }
    assert_and_release_unrelated_authority(unrelated, &disposed, unrelated_width);
    evidence
}
pub fn denied_readmission_work(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime) =
        crate::domain_computation::managed_run::tests::readmission_direct::yielded_direct(
            execution,
            resource_request,
        );
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width, resource_request);
    let foreign = query_runtime();
    let denial = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, &foreign, &bridge)
        };
        match fixture_phase_result {
            WorthQueryDirectReadmissionOutcome::Denied(denial) => denial,
            _ => panic!("foreign Query runtime should deny readmission"),
        }
    };
    let evidence = denial.readmission_evidence();
    complete_direct_yield_cleanup(denial.into_yielded());
    assert_and_release_unrelated_authority(unrelated, &disposed, unrelated_width);
    evidence
}
pub fn unrelated_authority(
    runtime: &WorthQueryExecutionRuntime,
    width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    Vec<(
        WorthQueryRunningWorkflowRun,
        worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
    Arc<AtomicUsize>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..width)
        .map(|index| {
            unrelated_artifact_run_in_runtime(
                runtime,
                index,
                Arc::clone(&disposed),
                resource_request,
            )
        })
        .collect();
    (unrelated, disposed)
}
pub fn assert_and_release_unrelated_authority<T>(
    unrelated: Vec<T>,
    disposed: &Arc<AtomicUsize>,
    expected_width: usize,
) {
    assert_eq!(unrelated.len(), expected_width);
    assert_eq!(disposed.load(Ordering::Acquire), 0);
    drop(unrelated);
    assert_eq!(disposed.load(Ordering::Acquire), expected_width);
}
pub fn assert_exact_direct_readmission_work(evidence: WorthQueryReadmissionEvidence) {
    let counters = evidence.query_counters();
    assert_eq!(counters.preflight_check_count(), 1);
    assert_eq!(counters.fresh_resource_attempt_count(), 1);
    assert_eq!(counters.bridge_readmission_attempt_count(), 1);
    assert_eq!(counters.provider_restore_attempt_count(), 1);
    assert_eq!(counters.artifact_generation_attempt_count(), 0);
    assert_eq!(counters.artifact_generation_commit_count(), 0);
    assert_eq!(counters.committed_attempt_count(), 1);
    assert_exact_committed_bridge_readmission_work(evidence);
}
pub fn assert_exact_workflow_readmission_work(evidence: WorthQueryReadmissionEvidence) {
    let counters = evidence.query_counters();
    assert_eq!(counters.preflight_check_count(), 1);
    assert_eq!(counters.fresh_resource_attempt_count(), 1);
    assert_eq!(counters.bridge_readmission_attempt_count(), 1);
    assert_eq!(counters.provider_restore_attempt_count(), 1);
    assert_eq!(counters.artifact_generation_attempt_count(), 1);
    assert_eq!(counters.artifact_generation_commit_count(), 1);
    assert_eq!(counters.committed_attempt_count(), 1);
    assert_exact_committed_bridge_readmission_work(evidence);
}
pub fn assert_exact_preflight_denial_work(evidence: WorthQueryReadmissionEvidence) {
    let counters = evidence.query_counters();
    assert_eq!(counters.preflight_check_count(), 1);
    assert_eq!(counters.fresh_resource_attempt_count(), 0);
    assert_eq!(counters.bridge_readmission_attempt_count(), 0);
    assert_eq!(counters.provider_restore_attempt_count(), 0);
    assert_eq!(counters.artifact_generation_attempt_count(), 0);
    assert_eq!(counters.artifact_generation_commit_count(), 0);
    assert_eq!(counters.committed_attempt_count(), 0);
    assert!(evidence.bridge_counters().is_none());
}
pub fn assert_exact_committed_bridge_readmission_work(evidence: WorthQueryReadmissionEvidence) {
    let counters = evidence
        .bridge_counters()
        .expect("successful readmission must carry Bridge owner evidence");
    assert_eq!(counters.preflight_check_count(), 1);
    assert_eq!(counters.reservation_check_count(), 1);
    assert_eq!(counters.signal_attempt_admission_count(), 1);
    assert_eq!(counters.signal_attempt_check_count(), 1);
    assert_eq!(counters.signal_queue_binding_count(), 1);
    assert_eq!(counters.abort_count(), 0);
    assert_eq!(counters.commit_count(), 1);
}
pub fn assert_exact_admission_work(
    counters: &worth_query_execution::facade::domain_computation::WorthQueryManagedRunCounters,
) {
    assert_eq!(counters.query_runtime_check_count(), 1);
    assert_eq!(counters.resource_attempt_check_count(), 1);
    assert_eq!(counters.bridge_intent_check_count(), 1);
    assert_eq!(counters.bridge_source_check_count(), 1);
    assert_eq!(counters.relational_basis_check_count(), 1);
    assert_eq!(counters.semantic_basis_check_count(), 1);
}
pub fn assert_rejection_work_is_constant_with_unrelated_authority(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    let owner = query_runtime();
    let foreign = query_runtime();
    let plan = admitted_plan("cost-bound-rejection", 8);
    let operation = direct_authority(&owner, &plan);
    let attempt = owner
        .start_direct_resource_attempt(&operation, plan)
        .expect("rejection target should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
        resource_request,
    );
    let rejection =
        match foreign.admit_direct_run_for_test(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("foreign runtime admitted another runtime's run"),
            Err(rejection) => rejection,
        };
    assert_eq!(rejection.denial().counters().query_runtime_check_count(), 1);
    assert_eq!(
        rejection.denial().counters().resource_attempt_check_count(),
        0
    );
    assert_eq!(rejection.denial().counters().bridge_intent_check_count(), 0);
    assert_eq!(rejection.denial().counters().bridge_source_check_count(), 0);
    assert_eq!(
        rejection.denial().counters().relational_basis_check_count(),
        0
    );
    assert_eq!(
        rejection.denial().counters().semantic_basis_check_count(),
        0
    );
}
pub struct CostBoundArtifactResource(Arc<AtomicUsize>);
impl WorthQueryArtifactProviderResource for CostBoundArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"unrelated-managed-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        32
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
