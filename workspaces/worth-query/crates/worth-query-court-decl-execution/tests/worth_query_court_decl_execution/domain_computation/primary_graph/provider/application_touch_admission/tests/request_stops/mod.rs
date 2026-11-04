//! Original request stops retain their typed causes and committed truth.
use crate::domain_computation::primary_graph::provider::application_touch_admission::tests::*;
use worth_execution::CancellationSource;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind as Kind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;

#[test]
fn touch_projection_preserves_request_stops_and_committed_truth() {
    check(|execution| {
        let world = World::new(execution, false, &[("A", "sentinel"), ("B", "protected")]);
        let candidate = world.candidate(execution, [world.clear()]).unwrap();
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let payload = world.truth(execution);
        let source = CancellationSource::new();
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(execution.memory_limit()),
            source.token(),
            None,
        );
        source.cancel();
        let stopped = world
            .admit(&ExecutionRequest::serial(&host), &candidate, true)
            .unwrap_err()
            .into_invariant_failure_for_test();
        assert_eq!(
            stopped.kind(),
            Kind::ExecutionControlStopped(Control::Cancelled)
        );
        let expired = SerialRequest::from_memory(
            SerialMemoryBudget::new(execution.memory_limit()),
            CancellationSource::new().token(),
            Some({
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            }),
        );
        let stopped = world
            .admit(&ExecutionRequest::serial(&expired), &candidate, true)
            .unwrap_err()
            .into_invariant_failure_for_test();
        assert_eq!(
            stopped.kind(),
            Kind::ExecutionControlStopped(Control::TimedOut)
        );
        let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let original = ExecutionRequest::serial(&host);
        let (stopped, _) = worth_execution::ExecutionWorkCeiling::new(Some(1))
            .run_serial(&host, || world.admit(&original, &candidate, true))
            .expect("the original request enters its deliberately narrow work scope");
        let stopped = stopped.unwrap_err().into_invariant_failure_for_test();
        assert!(matches!(stopped.kind(), Kind::ExecutionDenied(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted, .. })), "typed actual work stop: {stopped:?}");
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), payload);
    });
}
#[test]
fn touch_projection_preserves_memory_refusal_and_committed_truth() {
    check(|execution| {
        let world = World::new(execution, false, &[("A", "sentinel"), ("B", "protected")]);
        let candidate = world.candidate(execution, [world.clear()]).unwrap();
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let payload = world.truth(execution);
        // Zero admitted memory is an intentional caller refusal, not a new
        // production quota. No projection backing or effects may escape it.
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(0)),
            CancellationSource::new().token(),
            None,
        );
        let stopped = world
            .admit(&ExecutionRequest::serial(&host), &candidate, true)
            .unwrap_err()
            .into_invariant_failure_for_test();
        assert!(
            matches!(stopped.kind(), Kind::ExecutionDenied(
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit { requested: 1.., admitted: 0, .. }, .. })),
            "typed native resource stop: {stopped:?}"
        );
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), payload);
    });
}
