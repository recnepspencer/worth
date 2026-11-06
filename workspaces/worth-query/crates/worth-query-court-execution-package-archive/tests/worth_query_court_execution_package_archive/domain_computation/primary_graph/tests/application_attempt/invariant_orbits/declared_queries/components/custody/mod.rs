//! Retained original bodies distinguish Native membership from physical dependencies.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::components::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection::ComponentSelection;
use std::sync::Arc;
use worth_execution::CancellationSource;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub fn body<Q>(
    world: &World,
    source: &WorthQueryObservedSource<Q>,
    request: &WorthQueryRequestScope,
) -> Arc<ComponentSelection> {
    let layout = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test();
    let facts = source
        .retained_checkpoint_facts_for_test(layout, Some(request), Allocation::SystemAllocation)
        .unwrap();
    let mut bodies = facts.iter().filter_map(|fact| match fact {
        WorthQueryApplicationObservedFact::ComponentSelection { observation } => {
            Some(Arc::clone(observation))
        }
        _ => None,
    });
    let selected = bodies
        .next()
        .expect("component source retains its original body");
    assert!(bodies.next().is_none());
    selected
}
#[test]
fn component_rows_share_cohort_custody_and_census_retains_complete_set_truth() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = interleaved_world();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let denied_payer = SerialMemoryBudget::new(Some(0));
    let denied_host = SerialRequest::from_memory(
        denied_payer.clone(),
        CancellationSource::new().token(),
        None,
    );
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(
        &world,
        &ExecutionRequest::serial(&denied_host),
        &request,
        Census::reference(),
        0,
    );
    let Err(worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Execution(denial)) = denied else {
        panic!("zero original memory must refuse without a completed result");
    };
    use worth_execution::LeaseDenial;
    use worth_execution::MemoryLimitDenial;
    use worth_execution::MemoryLimitLevel;
    use worth_execution::WorkCeilingDenial;
    assert!(matches!(
        denial,
        WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
            requested,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        })) if requested > 0
    ));
    assert_eq!(denied_payer.charged_memory_bytes(), 0);
    // This is the earliest actual read allocation, not a particular later
    // material-member allocation or a fitted byte ceiling.
    let payer = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(payer.clone(), CancellationSource::new().token(), None);
    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(
        &world,
        &ExecutionRequest::serial(&host),
        &request,
        Census::reference(),
        0,
    )
    .unwrap();
    let retained_source = result.observed_sources()[0].clone();
    let full = body(&world, result.result_set_observation().source(), &request);
    assert_eq!(full.aggregates().len(), 2);
    assert!(
        full.operation_scope().is_none(),
        "Query bodies cannot seal as issued operation reads"
    );
    let rows = result
        .observed_sources()
        .iter()
        .map(|source| body(&world, source, &request))
        .collect::<Vec<_>>();
    let mut offset = 0;
    for cohort in full.aggregates() {
        let count = cohort.native().len();
        assert!(!Arc::ptr_eq(&rows[offset], &full));
        for row in &rows[offset..offset + count] {
            assert!(Arc::ptr_eq(row, &rows[offset]));
            assert!(Arc::ptr_eq(&row.aggregates()[0], cohort));
        }
        offset += count;
    }
    assert!(!Arc::ptr_eq(&rows[0], rows.last().unwrap()));
    // Keep both independently funded Native payloads constant while releasing
    // the Query wrappers and their material-selection backing.
    let native = [
        full.aggregates()[0].native().clone(),
        full.aggregates()[1].native().clone(),
    ];
    drop(result);
    drop(retained_source);
    let with_material_selections = payer.charged_memory_bytes();
    assert!(with_material_selections > 0);
    drop(full);
    drop(rows);
    let native_only = payer.charged_memory_bytes();
    assert!(native_only > 0);
    assert!(
        native_only < with_material_selections,
        "last material selection releases its own original-request storage"
    );
    drop(native);
    assert_eq!(payer.charged_memory_bytes(), 0);

    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Of::reference(), 0);
    let full = body(&world, result.result_set_observation().source(), &request);
    assert_eq!(full.aggregates().len(), 1);
    for source in result.observed_sources() {
        assert!(Arc::ptr_eq(&body(&world, source, &request), &full));
    }
}
#[test]
fn query_component_currentness_includes_coupling_even_when_native_membership_returns() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let originals = [0, 3].map(|root| {
        let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Of::reference(), root);
        body(&world, result.result_set_observation().source(), &request)
    });
    let winner = usize::from(
        originals[0].aggregates()[0].native().anchor()
            > originals[1].aggregates()[0].native().anchor(),
    );
    let original = &originals[winner];
    for key in [249, 250] {
        let edit = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::commit(&world, &request, edit, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    let fresh = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Of::reference(), if winner == 0 { 0 } else { 3 });
    let fresh = body(&world, fresh.result_set_observation().source(), &request);
    assert_eq!(
        original.aggregates()[0].native().identity(),
        fresh.aggregates()[0].native().identity()
    );
    assert_eq!(
        original.aggregates()[0]
            .native()
            .members()
            .collect::<Vec<_>>(),
        fresh.aggregates()[0].native().members().collect::<Vec<_>>()
    );
    let lease = crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::currentness::snapshot(&world, &execution);
    lease.handle_for_test().with_runtime(&execution, |runtime| {
        assert!(original.aggregates()[0]
            .remains_equal_for_test(runtime, &execution, lease.snapshot_for_test(), None)
            .unwrap());
        assert!(!original
            .remains_equal_for_test(runtime, &execution, lease.snapshot_for_test(), None)
            .unwrap());
    });
}
#[test]
fn empty_query_census_retains_negative_guard_and_ignores_unrelated_field_changes() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Empty::reference(), 0);
    let original = body(&world, result.result_set_observation().source(), &request);
    assert!(original.aggregates().is_empty());
    for (root, key, expected) in [
        (6, "query-components-unrelated", true),
        (0, "query-components-guard", false),
    ] {
        let (candidate, idempotency) =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::prepare(&world, &request, root, key);
        assert!(matches!(
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::publish(
                &world,
                &request,
                candidate,
                idempotency
            ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let lease = crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::currentness::snapshot(&world, &execution);
        assert_eq!(
            lease
                .handle_for_test()
                .with_runtime(&execution, |runtime| original
                    .remains_equal_for_test(runtime, &execution, lease.snapshot_for_test(), None)
                    .unwrap()),
            expected
        );
    }
}
