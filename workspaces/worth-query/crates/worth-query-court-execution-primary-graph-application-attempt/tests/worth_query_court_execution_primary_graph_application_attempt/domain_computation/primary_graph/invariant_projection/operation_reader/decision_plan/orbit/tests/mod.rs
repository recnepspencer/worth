//! A refusal after aggregate registration cannot seal the partially recorded pair.
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionTraversalDenialKind;
use worth_execution::ExecutionAllocationPolicy as Allocation;
#[test]
fn failure_of_second_orbit_registration_vetoes_the_partial_pair() {
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let root = reader
                            .resolve_entity(NodeKey::at("NodeKey"), 0_u64)
                            .unwrap();
                        reader.orbit(Next::reference(), &root).unwrap()
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (earlier, _snapshot, _) = completed.into_parts();
    world.application.with_host_advancement(|phase| {
        let host = phase.test_host_request_for_test();
        let execution = host.execution_request_for_for_test(world.application.product_runtime()).unwrap();
        let (admission,_) = authorize(&world, &phase, &execution, &request, 0, ReplaceNext::reference());
        let (_, projection, _) = world.invariant.project_admitted_operation(&phase, &admission, |reader, root| {
            assert!(!earlier.walk().issued_at_for_test(&(*(*reader.reader_for_test()).native_for_test()).origin_for_test((*reader.reader_for_test()).snapshot_for_test()), (*(*reader.reader_for_test()).authority_identity_for_test())));
            let contracts = (*reader.admitted_contracts_for_test()).as_ref().unwrap().clone();
            let admitted = (*reader.admission_identity_for_test()).unwrap();
            // The deliberately conflicting registry value is a genuine earlier
            // Native body, never fabricated graph truth or a new construction door.
            let original = *(*(*reader.reader_for_test()).execution_for_test());
            let seeded = original.run_native_with_checkpoint(|context| (*reader.decision_facts_mut_for_test()).record_walk(earlier.walk(), admitted, contracts, &original, context).is_ok()).unwrap().0;
            assert!(seeded);
            let before = (*reader.reader_for_test()).work();
            let error = reader.decision_orbit(Next::reference(), root).unwrap_err();
            assert_eq!(error.kind(), WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity);
            assert!((*reader.reader_for_test()).work().adjacency_lists_read() > before.adjacency_lists_read(), "the actual complete orbit was read before its second registration refused");
        }, Allocation::SystemAllocation).unwrap().into_parts();
        let error = world.application.begin_projected_application_read_attempt(&phase, admission, projection, Allocation::SystemAllocation).err().unwrap();
        assert_eq!(error.kind(), WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, "swallowing a second-registration failure cannot seal the first fact alone");
    }).unwrap();
}
