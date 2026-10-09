use super::*;
use crate::domain_computation::{
    WorthQueryDirectReadmissionOutcome, WorthQueryReadmissionEvidence,
    WorthQueryRunningWorkflowRun, WorthQueryWorkflowReadmissionOutcome,
};

const UNRELATED_RUN_COUNT: usize = 12;

#[test]
fn managed_run_work_is_invariant_to_unrelated_live_authority_width() {
    let disposed = Arc::new(AtomicUsize::new(0));
    let runtime = query_runtime();
    let unrelated = (0..UNRELATED_RUN_COUNT)
        .map(|index| unrelated_artifact_run_in_runtime(&runtime, index, Arc::clone(&disposed)))
        .collect::<Vec<_>>();

    let plan = admitted_plan("cost-bound-target", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("target operation should start");
    let lower = causal_fixture::managed_admission_context();
    let admitted = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("target run should admit independently of unrelated authority width");

    assert_exact_admission_work(admitted.counters());
    let terminal = admitted
        .start()
        .completed()
        .expect("target run with no provider work should complete");
    assert_exact_admission_work(terminal.counters());
    assert_eq!(terminal.provider_work().issued_call_count(), 0);
    let cleanup = terminal.cleanup().expect("target cleanup should complete");
    assert_exact_admission_work(cleanup.inspection().counters());
    assert_eq!(cleanup.inspection().released_reservation_count(), 1);
    assert!(cleanup.inspection().resources_released());
    assert_eq!(disposed.load(Ordering::Acquire), 0);

    assert_rejection_work_is_constant_with_unrelated_authority();
    assert_eq!(disposed.load(Ordering::Acquire), 0);

    drop(unrelated);
    assert_eq!(disposed.load(Ordering::Acquire), UNRELATED_RUN_COUNT);
}

#[test]
fn readmission_work_is_invariant_to_same_runtime_unrelated_authority_width() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let direct_empty = direct_readmission_work(execution, 0);
        let direct_wide = direct_readmission_work(execution, UNRELATED_RUN_COUNT);
        assert_eq!(direct_empty, direct_wide);
        assert_exact_direct_readmission_work(direct_wide);

        let workflow_empty = workflow_readmission_work(execution, 0);
        let workflow_wide = workflow_readmission_work(execution, UNRELATED_RUN_COUNT);
        assert_eq!(workflow_empty, workflow_wide);
        assert_exact_workflow_readmission_work(workflow_wide);

        let denial_empty = denied_readmission_work(execution, 0);
        let denial_wide = denied_readmission_work(execution, UNRELATED_RUN_COUNT);
        assert_eq!(denial_empty, denial_wide);
        assert_exact_preflight_denial_work(denial_wide);
    });
}

pub(super) fn unrelated_artifact_run(
    index: usize,
    disposed: Arc<AtomicUsize>,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    crate::domain_computation::artifact_owner::WorthQueryMoveOnlyArtifactHandle,
) {
    let runtime = query_runtime();
    unrelated_artifact_run_in_runtime(&runtime, index, disposed)
}

fn unrelated_artifact_run_in_runtime(
    runtime: &WorthQueryExecutionRuntime,
    index: usize,
    disposed: Arc<AtomicUsize>,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    crate::domain_computation::artifact_owner::WorthQueryMoveOnlyArtifactHandle,
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
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation =
        workflow_authority_with_output_artifact(runtime, &resources, "producer", output);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("unrelated workflow should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request())
        .expect("unrelated workflow should admit")
        .start()
        .expect("unrelated workflow should start");
    let production = running
        .artifacts()
        .production_authority("producer")
        .expect("producer stage should validate")
        .expect("producer should own artifact authority");
    let admission =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
            &production,
            WorthQueryArtifactProductionEvidence::new(
                format!("unrelated-provenance-{index}"),
                format!("unrelated-dependency-{index}"),
            ),
        );
    let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            CostBoundArtifactResource(disposed),
        )
        .expect("unrelated artifact should register");
    (running, handle)
}

fn direct_readmission_work(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime) = super::readmission_direct::yielded_direct(execution);
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width);
    let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
        WorthQueryDirectReadmissionOutcome::Readmitted(readmitted) => readmitted,
        _ => panic!("direct cost target should readmit"),
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

fn workflow_readmission_work(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime, old_producer) = super::readmission_workflow::yielded_workflow(
        execution,
        super::yield_fixture::YieldProvider::installed(7),
    );
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width);
    let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
        WorthQueryWorkflowReadmissionOutcome::Readmitted(readmitted) => readmitted,
        _ => panic!("workflow cost target should readmit"),
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

fn denied_readmission_work(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
) -> WorthQueryReadmissionEvidence {
    let active_request = execution;

    let (yielded, bridge, runtime) = super::readmission_direct::yielded_direct(execution);
    let (unrelated, disposed) = unrelated_authority(&runtime, unrelated_width);
    let foreign = query_runtime();
    let denial = match yielded.readmit_same_runtime(active_request, &foreign, &bridge) {
        WorthQueryDirectReadmissionOutcome::Denied(denial) => denial,
        _ => panic!("foreign Query runtime should deny readmission"),
    };
    let evidence = denial.readmission_evidence();
    complete_direct_yield_cleanup(denial.into_yielded());
    assert_and_release_unrelated_authority(unrelated, &disposed, unrelated_width);
    evidence
}

fn unrelated_authority(
    runtime: &WorthQueryExecutionRuntime,
    width: usize,
) -> (
    Vec<(
        WorthQueryRunningWorkflowRun,
        crate::domain_computation::artifact_owner::WorthQueryMoveOnlyArtifactHandle,
    )>,
    Arc<AtomicUsize>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..width)
        .map(|index| unrelated_artifact_run_in_runtime(runtime, index, Arc::clone(&disposed)))
        .collect();
    (unrelated, disposed)
}

fn assert_and_release_unrelated_authority<T>(
    unrelated: Vec<T>,
    disposed: &Arc<AtomicUsize>,
    expected_width: usize,
) {
    assert_eq!(unrelated.len(), expected_width);
    assert_eq!(disposed.load(Ordering::Acquire), 0);
    drop(unrelated);
    assert_eq!(disposed.load(Ordering::Acquire), expected_width);
}

fn assert_exact_direct_readmission_work(evidence: WorthQueryReadmissionEvidence) {
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

fn assert_exact_workflow_readmission_work(evidence: WorthQueryReadmissionEvidence) {
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

fn assert_exact_preflight_denial_work(evidence: WorthQueryReadmissionEvidence) {
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

fn assert_exact_committed_bridge_readmission_work(evidence: WorthQueryReadmissionEvidence) {
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

pub(super) fn assert_exact_admission_work(counters: &super::super::WorthQueryManagedRunCounters) {
    assert_eq!(counters.query_runtime_check_count(), 1);
    assert_eq!(counters.resource_attempt_check_count(), 1);
    assert_eq!(counters.bridge_intent_check_count(), 1);
    assert_eq!(counters.bridge_source_check_count(), 1);
    assert_eq!(counters.relational_basis_check_count(), 1);
    assert_eq!(counters.semantic_basis_check_count(), 1);
}

fn assert_rejection_work_is_constant_with_unrelated_authority() {
    let owner = query_runtime();
    let foreign = query_runtime();
    let plan = admitted_plan("cost-bound-rejection", 8);
    let operation = direct_authority(&owner, &plan);
    let attempt = owner
        .start_direct_resource_attempt(&operation, plan)
        .expect("rejection target should start");
    let lower = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );
    let rejection =
        match foreign.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
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

struct CostBoundArtifactResource(Arc<AtomicUsize>);

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
