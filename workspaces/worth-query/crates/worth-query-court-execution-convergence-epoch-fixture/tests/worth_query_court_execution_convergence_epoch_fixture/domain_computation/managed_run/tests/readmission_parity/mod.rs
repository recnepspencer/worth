pub use std::collections::BTreeSet;

pub use crate::domain_computation::managed_run::tests::*;
pub use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
pub use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters;
pub use worth_query_admission::integration::admit_execution_resource_plan;
pub use worth_query_admission::integration::reserve_execution_resource_plan;
pub use worth_query_execution::facade::domain_computation::WorthQueryProviderWorkReport;
pub use worth_query_installation::facade::WorthQueryExecutionProviderRequirements;
pub use worth_query_installation::facade::WorthQueryExecutionResourceContract;
pub use worth_query_installation::facade::WorthQueryExecutionStrategyContract;
pub use worth_query_installation::facade::WorthQueryExecutionStrategyName;

pub use crate::domain_computation::managed_run::tests::readmission_parity::repeated_yield_provider::RepeatedYieldProvider;

#[test]
fn repeated_readmission_matches_uninterrupted_semantics_and_structural_evidence() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let uninterrupted = execute_repeated_yield_world(execution, false, resource_request);
        let resumed = execute_repeated_yield_world(execution, true, resource_request);

        assert_eq!(resumed.readmission_count, 2);
        assert_eq!(resumed.unique_managed_attempt_count, 3);
        assert_eq!(resumed.unique_resource_attempt_count, 3);
        assert_eq!(resumed.unique_bridge_request_count, 3);
        assert_eq!(resumed.capacity_reservation_count, 2);
        assert_eq!(resumed.provider_receipt, uninterrupted.provider_receipt);
        assert_eq!(resumed.work_report, uninterrupted.work_report);
        assert_eq!(resumed.provider_work, uninterrupted.provider_work);
        assert_eq!(resumed.run_counters, uninterrupted.run_counters);
    });
}

#[test]
fn readmission_transfers_saturated_capacity_without_a_second_reservation() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, mut probes) =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_direct::yielded_direct_with_plan_observation(
                execution,
                |resources| {
                    (0..11)
                        .map(|_| readmit_capacity_probe(resources))
                        .collect::<Vec<_>>()
                },
                resource_request,
            );
        let mut saturation_holders = Vec::new();
        for probe in probes.drain(..8) {
            let Some(reservation) = reserve_execution_resource_plan(probe) else {
                break;
            };
            saturation_holders.push(reservation);
        }
        let saturation_probe = probes.remove(0);
        let active_probe = probes.remove(0);
        let released_probe = probes.remove(0);
        let retained_reservations = yielded.inspection().retained_capacity_reservation_count();

        assert_eq!(
            saturation_holders.len(),
            7,
            "the yielded run plus seven arrivals must fill the eight-slot pools"
        );
        assert!(
            reserve_execution_resource_plan(saturation_probe).is_none(),
            "arrival pressure must saturate every capacity pool"
        );
        let active = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("same-runtime readmission must transfer retained capacity"),
            }
        };
        assert_eq!(
            active.retained_capacity_reservation_count(),
            retained_reservations
        );
        assert!(
            reserve_execution_resource_plan(active_probe).is_none(),
            "readmission must not release or duplicate the retained reservation"
        );

        let completion = {
            let fixture_phase_result = {
                let installed_fixture_phase = active.fixture_phase_for_test(execution);
                active.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("restored provider should complete"),
            }
        };
        let cleanup = completion
            .into_running()
            .completed()
            .expect("completed execution should terminalize")
            .cleanup()
            .expect("completed readmitted run should clean up");
        assert_eq!(
            cleanup.inspection().released_reservation_count(),
            retained_reservations
        );

        let released_capacity = reserve_execution_resource_plan(released_probe)
            .expect("terminal cleanup must return one slot in every retained capacity pool");
        drop(released_capacity);
        drop(saturation_holders);
    });
}

pub fn readmit_capacity_probe(
    resources: &WorthQueryAdmittedExecutionResourcePlan,
) -> WorthQueryAdmittedExecutionResourcePlan {
    let executor = resources.support_snapshot().executor();
    let strategy = WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new(resources.strategy().as_str()).unwrap(),
        resources.envelope().clone(),
        WorthQueryExecutionProviderRequirements::new(
            executor.provider().clone(),
            executor.access_product().clone(),
            executor.allocator().clone(),
        ),
    );
    let contract = WorthQueryExecutionResourceContract::declared([strategy]).unwrap();
    admit_execution_resource_plan(
        resources.binding_identity(),
        &contract,
        resources.request(),
        resources.support_snapshot().clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .expect("equivalent saturation probe should be independently admitted")
}

pub struct ParityEvidence {
    pub provider_receipt: String,
    pub work_report: WorthQueryProviderWorkReport,
    pub provider_work: ProviderWorkEvidence,
    pub run_counters:
        worth_query_execution::facade::domain_computation::WorthQueryManagedRunCounters,
    pub readmission_count: usize,
    pub unique_managed_attempt_count: usize,
    pub unique_resource_attempt_count: usize,
    pub unique_bridge_request_count: usize,
    pub capacity_reservation_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProviderWorkEvidence {
    pub completed_work_units: u64,
    pub applied_effect_count: u64,
    pub produced_artifact_count: usize,
    pub retained_artifact_count: usize,
    pub disposed_artifact_count: usize,
    pub peak_scratch_bytes: usize,
    pub retained_bytes: usize,
    pub provider_step_attempt_count: usize,
    pub safe_point_request_lookup_count: usize,
    pub pressure_classification_count: usize,
    pub output_capacity_classification_count: usize,
    pub queue_state_mutation_count: usize,
}

pub fn execute_repeated_yield_world(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resume_every_safe_point: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> ParityEvidence {
    let active_request = execution;

    let (running, graph, bridge, runtime) = managed_graph_run_with_provider_and_runtime(
        WorthQueryOperationGraphAccess::Observe,
        RepeatedYieldProvider { step_count: 3 },
        resource_request,
    );
    let installed_running_phase = execution.phase_for_direct_for_test(&running);

    let active = running
        .begin_graph_execution(
            &installed_running_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "repeated-yield-parity",
            ),
        )
        .expect("parity provider should begin");
    let mut managed_attempts = BTreeSet::from([active.run_identity().to_owned()]);
    let mut resource_attempts = BTreeSet::from([active.resource_attempt_identity().to_owned()]);
    let mut bridge_requests = BTreeSet::from([active.bridge_request_identity().to_owned()]);
    let capacity_reservation_count = active.retained_capacity_reservation_count();
    let mut readmission_count = 0;
    let mut outcome = active.advance(&installed_running_phase);

    loop {
        outcome = match outcome {
            WorthQueryDirectGraphStepOutcome::Continue(paused) if resume_every_safe_point => {
                let yielded = match paused.yield_run() {
                    worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => {
                        yielded
                    }
                    _ => panic!("parity safe point should yield"),
                };
                let active = {
                    let fixture_phase_result = {
                        let installed_fixture_phase =
                            yielded.fixture_phase_for_test(active_request);
                        yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
                    };
                    match fixture_phase_result {
                    worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                        readmitted,
                    ) => readmitted.into_active(),
                    _ => panic!("parity yielded run should readmit"),
                }
                };
                readmission_count += 1;
                managed_attempts.insert(active.run_identity().to_owned());
                resource_attempts.insert(active.resource_attempt_identity().to_owned());
                bridge_requests.insert(active.bridge_request_identity().to_owned());
                assert_eq!(
                    active.retained_capacity_reservation_count(),
                    capacity_reservation_count
                );
                active.advance(&installed_running_phase)
            }
            WorthQueryDirectGraphStepOutcome::Continue(paused) => {
                paused.advance(&installed_running_phase)
            }
            WorthQueryDirectGraphStepOutcome::Completed(completion) => {
                return parity_evidence(
                    completion,
                    managed_attempts,
                    resource_attempts,
                    bridge_requests,
                    readmission_count,
                    capacity_reservation_count,
                );
            }
            _ => panic!("parity provider should continue or complete"),
        };
    }
}

pub fn parity_evidence(
    completion: worth_query_execution::facade::domain_computation::WorthQueryCompletedDirectGraphExecution,
    managed_attempts: BTreeSet<String>,
    resource_attempts: BTreeSet<String>,
    bridge_requests: BTreeSet<String>,
    readmission_count: usize,
    capacity_reservation_count: usize,
) -> ParityEvidence {
    let provider_receipt = completion.receipt().provider_receipt().to_owned();
    let work_report = completion.receipt().work_report();
    let terminal = completion.into_running().completed().unwrap();
    let provider_work = terminal.provider_work();
    let evidence = ParityEvidence {
        provider_receipt,
        work_report,
        provider_work: ProviderWorkEvidence {
            completed_work_units: provider_work.completed_work_units(),
            applied_effect_count: provider_work.applied_effect_count(),
            produced_artifact_count: provider_work.produced_artifact_count(),
            retained_artifact_count: provider_work.retained_artifact_count(),
            disposed_artifact_count: provider_work.disposed_artifact_count(),
            peak_scratch_bytes: provider_work.peak_scratch_bytes(),
            retained_bytes: provider_work.retained_bytes(),
            provider_step_attempt_count: provider_work.provider_step_attempt_count(),
            safe_point_request_lookup_count: provider_work.safe_point_request_lookup_count(),
            pressure_classification_count: provider_work.pressure_classification_count(),
            output_capacity_classification_count: provider_work
                .output_capacity_classification_count(),
            queue_state_mutation_count: provider_work.queue_state_mutation_count(),
        },
        run_counters: terminal.counters().clone(),
        readmission_count,
        unique_managed_attempt_count: managed_attempts.len(),
        unique_resource_attempt_count: resource_attempts.len(),
        unique_bridge_request_count: bridge_requests.len(),
        capacity_reservation_count,
    };
    assert!(terminal.cleanup().is_ok());
    evidence
}

#[path = "repeated_yield_provider/mod.rs"]
pub mod repeated_yield_provider;
