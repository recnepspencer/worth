use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture;
use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_domain_port_probe;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::domain_port_probe::FixtureDomainPortProbe;
use crate::domain_computation::convergence_epoch::tests::terminal_fixture::workflow_indeterminate_terminal;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainInvocationFailureKind as FailureKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceDomainPhase as Phase;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceEpochDenialKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceIndeterminateCause;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunCleanupDisposition;
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind;

#[test]
fn query_counts_each_governed_domain_port_and_contains_rejection_or_panic() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let cases = [
            (
                FixtureDisposition::ComparatorFailure,
                Phase::Comparator,
                FailureKind::Rejected,
                [1, 0, 0],
            ),
            (
                FixtureDisposition::ComparatorPanic,
                Phase::Comparator,
                FailureKind::Panicked,
                [1, 0, 0],
            ),
            (
                FixtureDisposition::ProgressFailure,
                Phase::ProgressMeasure,
                FailureKind::Rejected,
                [1, 1, 0],
            ),
            (
                FixtureDisposition::ProgressPanic,
                Phase::ProgressMeasure,
                FailureKind::Panicked,
                [1, 1, 0],
            ),
            (
                FixtureDisposition::RepeatedStateFailure,
                Phase::RepeatedStateDetector,
                FailureKind::Rejected,
                [1, 1, 1],
            ),
            (
                FixtureDisposition::RepeatedStatePanic,
                Phase::RepeatedStateDetector,
                FailureKind::Panicked,
                [1, 1, 1],
            ),
        ];

        for (disposition, expected_phase, expected_kind, expected_work) in cases {
            let (terminal, provider_probe) =
                indeterminate_terminal_with_probe(execution, disposition, resource_request);
            assert_eq!(provider_probe.entries(), expected_work);
            assert_domain_invocation_cause(
                terminal.indeterminate_cause(),
                expected_phase,
                expected_kind,
                expected_work,
            );
            assert_eq!(
                [
                    terminal.counters().comparator_call_count(),
                    terminal.counters().progress_check_count(),
                    terminal.counters().repeated_state_probe_count(),
                ],
                expected_work
            );
            let cleanup = match terminal.cleanup() {
                Ok(cleanup) => cleanup,
                Err(_) => panic!("contained domain failure must retain direct cleanup authority"),
            };
            assert_domain_invocation_cause(
                cleanup.indeterminate_cause(),
                expected_phase,
                expected_kind,
                expected_work,
            );
            assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
            assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
        }
    });
}

pub fn indeterminate_terminal_with_probe(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceTerminal<
        worth_query_execution::court_doors_for_test::domain_computation::WorthQueryIndeterminate,
    >,
    FixtureDomainPortProbe,
) {
    let (fixture, probe) =
        direct_admission_fixture_with_domain_port_probe(disposition, resource_request);
    let epoch = fixture.admit();
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_iteration(
            &installed_fixture_phase,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "governed-domain-port-probe",
            ),
        )
    }
    .unwrap_or_else(|_| panic!("governed probe iteration must start"));
    let outcome = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("governed probe provider must complete"),
        }
    };
    let terminal = match outcome {
        WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("governed probe failure must remain indeterminate"),
    };
    (terminal, probe)
}

#[test]
fn workflow_cleanup_preserves_a_typed_governed_domain_panic() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let terminal = workflow_indeterminate_terminal(
            execution,
            FixtureDisposition::ProgressPanic,
            resource_request,
        );
        assert_domain_invocation_cause(
            terminal.indeterminate_cause(),
            Phase::ProgressMeasure,
            FailureKind::Panicked,
            [1, 1, 0],
        );

        let WorthQueryWorkflowConvergenceCleanupOutcome::Complete(cleanup) = terminal.cleanup()
        else {
            panic!("contained domain panic must retain workflow cleanup authority");
        };
        assert_domain_invocation_cause(
            cleanup.indeterminate_cause(),
            Phase::ProgressMeasure,
            FailureKind::Panicked,
            [1, 1, 0],
        );
    });
}

#[test]
fn provider_family_inspection_panic_denies_and_returns_every_admission_authority() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let fixture =
        direct_admission_fixture(FixtureDisposition::FamilyInspectionPanic, resource_request);
    let rejection = match fixture.runtime.admit_direct_convergence_epoch(
        &fixture.operation,
        fixture.contract,
        fixture.managed,
        fixture.graph,
    ) {
        Ok(_) => panic!("provider family panic must deny convergence admission"),
        Err(rejection) => rejection,
    };
    assert_eq!(
        rejection.denial().kind(),
        WorthQueryConvergenceEpochDenialKind::ConvergenceProviderFamilyInspectionPanicked
    );
    assert_eq!(
        rejection.denial().counters().graph_authority_check_count(),
        1
    );
    let (_, managed, _) = rejection.into_parts();
    let terminal = managed
        .start()
        .terminate_for_convergence_for_test(WorthQueryManagedRunTerminalKind::Failed);
    assert_eq!(terminal.provider_work().provider_step_attempt_count(), 0);
    let cleanup = match terminal.cleanup() {
        Ok(cleanup) => cleanup,
        Err(_) => panic!("rejected convergence admission must return managed cleanup authority"),
    };
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
}

pub fn assert_domain_invocation_cause(
    cause: Option<&WorthQueryConvergenceIndeterminateCause>,
    expected_phase: Phase,
    expected_kind: FailureKind,
    expected_work: [usize; 3],
) {
    let Some(WorthQueryConvergenceIndeterminateCause::DomainInvocation(failure)) = cause else {
        panic!("indeterminate terminal must carry typed domain invocation failure");
    };
    assert_eq!(failure.phase(), expected_phase);
    assert_eq!(failure.kind(), expected_kind);
    assert_eq!(
        [
            failure.work().comparator_call_count(),
            failure.work().progress_check_count(),
            failure.work().repeated_state_probe_count(),
        ],
        expected_work
    );
}
