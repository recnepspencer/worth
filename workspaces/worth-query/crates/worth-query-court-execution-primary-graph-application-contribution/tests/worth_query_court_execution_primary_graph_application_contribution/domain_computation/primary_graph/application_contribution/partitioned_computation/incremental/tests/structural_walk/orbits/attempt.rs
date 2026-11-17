//! Run under the supplied actual root request, then seal its retained computation.
use super::super::*;
use super::model::Installed;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationExecution;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_foundational::facade::DeterminismContract;
use worth_foundational::facade::ExecutionPosture;
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
pub fn run(
    world: &worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World,
    installed: &Installed,
    prior: Option<ComputationPrior>,
    expected: Run,
) -> Attempt {
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            authorize(
                world,
                &phase,
                &execution,
                &request,
                0,
                ReplaceNext::reference(),
            )
            .0
        })
        .unwrap();
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let execution = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(policy),
        &request,
    );
    assert!(
        !execution.has_accounting_for_test(),
        "these projected fixtures use an actual unaccounted request"
    );
    let native = execution.execution_request().unwrap();
    assert!(!native.has_accounting());
    worth_query_execution::facade::domain_computation::partitioned_computation_runs_on_this_thread_for_test();
    ComputationPrior::hand_in_test_for_test(prior);
    SealedComputationRun::keep_in_test_for_test(None);
    let mut outcome = None;
    installed.owner_for_test().gathered.lock().unwrap().clear();
    let completed = world
        .invariant
        .project_admitted_operation_with_request(
            &native,
            &admission,
            |reader, root| {
                let roots = vec![
                    root.clone(),
                    reader.resolve_entity(NodeKey::at("NodeKey"), 3).unwrap(),
                ];
                let prepared = installed
                    .prepare_through_for_test(reader, &execution, &roots)
                    .unwrap();
                let computed = prepared
                    .compute(WorthQueryManagedComputationExecution::new(&execution))
                    .unwrap();
                outcome = Some(computed.complete().unwrap());
            },
            Allocation::SystemAllocation,
        )
        .unwrap();
    ComputationPrior::hand_in_test_for_test(None);
    let (_, projection, _) = completed.into_parts();
    world.application.with_host_advancement(|phase| {
        let reads = world.application.begin_projected_application_read_attempt(
            &phase, admission, projection, Allocation::SystemAllocation,
        ).unwrap();
        let expected = reads.expected_facts_for_test().as_ref().unwrap();
        let admission = reads.admission_for_test();
        let issued: Vec<_> = expected.iter_for_test().filter_map(|key| {
            use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey as Key;
            match key {
                Key::StructuralWalk(key) => Some(expected.structural_walk_for_test(key, admission.admission_identity(), &admission.retain_graph_contract(), reads.lease_for_test().snapshot_for_test()).cloned().map(|observation| Fact::StructuralWalk { observation })),
                Key::Aggregate(key) => Some(expected.aggregate_for_test(key, admission.admission_identity(), &admission.retain_graph_contract(), reads.lease_for_test().snapshot_for_test()).cloned().map(|observation| Fact::Aggregate { observation })),
                _ => None,
            }
        }).collect();
        let completed = reads.complete_projected_dependencies(&phase, Allocation::SystemAllocation).unwrap();
        for body in issued {
            let body = body.expect("a successful seal consumed a currently issued structural body");
            assert!(completed.facts_for_test().iter_for_test().any(|fact| match (&body, fact) {
                (Fact::StructuralWalk { observation: a }, Fact::StructuralWalk { observation: b }) => Arc::ptr_eq(a, b),
                (Fact::Aggregate { observation: a }, Fact::Aggregate { observation: b }) => Arc::ptr_eq(a, b),
                _ => false,
            }), "projected seal consumes the currently issued walk and aggregate Arcs");
        }
    }).unwrap();
    let runs =
        worth_query_execution::facade::domain_computation::partitioned_computation_runs_on_this_thread_for_test();
    assert!(
        matches!(runs.as_slice(), [(actual, _)] if *actual == expected),
        "the actual comparator chose {expected:?}: {runs:?}"
    );
    Attempt {
        sealed: SealedComputationRun::kept_in_test_for_test()
            .expect("the real completed read sealed the owner run"),
        outcome: outcome.unwrap(),
        gathered: installed.owner_for_test().gathered.lock().unwrap().clone(),
    }
}

pub struct Attempt {
    pub sealed: SealedComputationRun,
    pub outcome: u64,
    pub gathered: Vec<u64>,
}
