use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::full_verification::FullVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

#[test]
fn a_retired_row_absent_in_both_images_does_not_change_the_image() {
    let _oracle =
        worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    retirement_image(request_execution, true);
}

#[test]
fn a_row_disappearing_after_capture_changes_the_image() {
    let _oracle =
        worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    retirement_image(request_execution, false);
}

pub fn retirement_image(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    retired_before_capture: bool,
) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected.resolve_entity(
                &phase,
                AccountStatus::at("AccountStatus"),
                "open".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
        })
        .expect("installed host call opens")
        .unwrap()
        .entity_id_for_test();
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("StatusOutput"));
    let identity = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let (snapshot, basis) = snapshot(runtime);
        register(
            owner,
            Arc::clone(&identity),
            Arc::from([]),
            &basis,
            OrdSet::new(),
        );
        let retire = || {
            assert_eq!(
                owner.retire_settlements_for_test(std::slice::from_ref(&identity)),
                vec![Arc::clone(&identity)]
            )
        };

        if retired_before_capture {
            retire();
        }
        let captured = owner
            .capture_full_verification_for_test(runtime, &snapshot, &basis)
            .unwrap();

        if !retired_before_capture {
            retire();
        }
        let observed = OrdSet::unit(Arc::clone(&identity));
        assert_eq!(
            captured.fence_semantic_image_for_test(&observed),
            if retired_before_capture {
                Ok(())
            } else {
                Err(FullVerificationStop::ActorImageChanged)
            }
        );
        register(owner, identity, Arc::from([]), &basis, OrdSet::new());
        assert_eq!(
            captured.fence_semantic_image_for_test(&observed),
            Err(FullVerificationStop::ActorImageChanged)
        );
        drop(captured);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
