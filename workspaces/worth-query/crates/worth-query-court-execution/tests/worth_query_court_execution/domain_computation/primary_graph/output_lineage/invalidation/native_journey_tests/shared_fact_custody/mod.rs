//! Independent consumed readers and restored marks share one original fact allocation.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;

#[test]
fn shared_original_facts_survive_restored_prerequisite_custody() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request_for_test();
        let execution = custody.borrowed_request_for_test();
        let entity = resolved_account(&custody.test_host_request_for_test(), &world, "open", &live_scope())
            .entity_id_for_test();
        let (_, product, _) = world.selected_product(&custody.test_host_request_for_test()).into_parts_for_test();
        let coordinate = ProductCoordinate::new_for_test(product.observation_for_test().lifecycle_incarnation(), product.observation_for_test().reference_generation().get());
        let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let handle = graph.integration_handle_for_test();
        let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
        let status = AccountStatus::reference();
        let status = graph.layout_for_test().field_locator_for_test(status.entity(), status.aspect(), status.field())
            .unwrap().clone();
        let (snapshot_handle, basis, identity, _facts, first, second) = handle.with_runtime_for_test(&execution, |runtime| {
            let (snapshot_handle, basis) = snapshot(runtime);
            let basis = Arc::new(basis);
            let identity = RecordedSettlementIdentity::retain_for_test(&SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("StatusOutput")), coordinate, 0);
            let facts = RetainedSourceFacts::for_test_for_test(false, Arc::from([
                field_fact(&execution, runtime, &snapshot_handle, entity, status),
            ])).for_comparison().unwrap();
            let witness = crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::request_recording::account_witness(
                &execution, &world, runtime, &snapshot_handle, entity,
            );
            assert_eq!(ConsumedOutputEvidence::verify_at_observation_for_test(
                &execution, &identity, &facts, &[],
                Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason::CheckpointRestore),
                &witness, owner, runtime, &snapshot_handle, &basis,
            ).unwrap(), ConsumedOutputVerification::Current);


            let first = ConsumedOutputEvidence::restored_with_witness_for_test(Arc::clone(&identity), facts.facts_for_test().clone(), Arc::clone(&basis), Arc::clone(&witness));


            let second = ConsumedOutputEvidence::restored_with_witness_for_test(Arc::clone(&identity), facts.facts_for_test().clone(), Arc::clone(&basis), witness);

            (snapshot_handle, basis, identity, facts, first, second)
        });
        first.establish_restored_before_commit_for_test(&execution, &(*handle.source_owner_for_test())).unwrap();
            assert!(matches!(owner.currentness_for_test(&basis, &identity).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert!(first.claim_verified_checkpoint_root_for_test(owner).unwrap(),
                "actual restored prerequisite custody must be established, not bypassed");
        handle.with_runtime_for_test(&execution, |runtime| {
            assert_eq!(ConsumedOutputEvidence::verify_many_with_admission_for_test(
                &execution, &[first.clone(), second.clone()], owner, runtime, &snapshot_handle,
                &basis,
            ).unwrap(), ConsumedOutputVerification::Current);
            let (current_handle, current_basis) = snapshot(runtime);
            assert_eq!(current_basis, *basis,
                "custody and currentness do not publish another native head");
            runtime.snapshots().release_snapshot(&current_handle).unwrap();
        });
            drop(first);
            drop(second);

            assert!(matches!(owner.currentness_for_test(&basis, &identity).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert_eq!(owner.retire_settlements_for_test(std::slice::from_ref(&identity)),
                vec![identity]);

        handle.with_runtime_for_test(&execution, |runtime| {
            runtime.snapshots().release_snapshot(&snapshot_handle).unwrap();
        });
    }).expect("the native custody journey owns its installed host call");
}
