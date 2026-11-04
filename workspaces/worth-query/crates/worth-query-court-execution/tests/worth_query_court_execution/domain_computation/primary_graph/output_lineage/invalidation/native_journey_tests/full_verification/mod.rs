//! Real Native probes and immutable pre-verification capture; no synthetic stamp.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::full_verification::FullVerificationDecision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::full_verification::FullVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn full_verification_uses_native_facts_and_retains_the_exact_captured_image() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _oracle =
        worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected.resolve_entity(
                &phase,
                AccountStatus::reference(),
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
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("LabelOutput"));
    let identity = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let (old_handle, old_basis) = snapshot(runtime);
        use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
        use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;

        assert_eq!(
            ConsumedOutputEvidence::verify_many_with_admission_for_test(
                request_execution,
                &[],
                owner,
                runtime,
                &old_handle,
                &old_basis
            ),
            Ok(ConsumedOutputVerification::Current),
            "an empty closure needs no actor image or fact authority"
        );
        register(
            owner,
            identity.clone(),
            Arc::from([field_fact(
                request_execution,
                runtime,
                &old_handle,
                entity,
                label.clone(),
            )]),
            &old_basis,
            OrdSet::new(),
        );
        let old = owner
            .capture_full_verification_for_test(runtime, &old_handle, &old_basis)
            .unwrap();

        assert_eq!(
            old.verify_root_for_test(request_execution, &identity, None),
            Err(FullVerificationStop::OutputEvidenceUnavailable),
            "source-only test registration cannot manufacture full output evidence"
        );
        drop(old);
        write_field(
            request_execution,
            runtime,
            entity,
            label.clone(),
            "changed-for-full-oracle",
        );
        let (changed_handle, changed_basis) = snapshot(runtime);
        let captured = owner
            .capture_full_verification_for_test(runtime, &changed_handle, &changed_basis)
            .unwrap();

        assert_eq!(
            captured.verify_root_for_test(request_execution, &identity, None),
            Ok(FullVerificationDecision::Changed)
        );
        // A later same-position registration replaces the actor's row. The
        // captured oracle still owns the earlier facts and cannot read that edit.
        register(
            owner,
            identity.clone(),
            Arc::from([field_fact(
                request_execution,
                runtime,
                &changed_handle,
                entity,
                label,
            )]),
            &changed_basis,
            OrdSet::new(),
        );
        assert!(matches!(
            currentness(owner, &changed_basis, &identity),
            SourceSettlementCurrentness::Clean
        ));
        assert_eq!(
            captured.verify_root_for_test(request_execution, &identity, None),
            Err(FullVerificationStop::ActorImageChanged),
            "an intervening actor edit cannot manufacture an equivalence mismatch"
        );
        let fresh = owner
            .capture_full_verification_for_test(runtime, &changed_handle, &changed_basis)
            .unwrap();

        assert_eq!(
            fresh.verify_root_for_test(request_execution, &identity, None),
            Err(FullVerificationStop::OutputEvidenceUnavailable)
        );
        assert!(
            owner
                .capture_full_verification_for_test(runtime, &old_handle, &changed_basis)
                .is_err(),
            "Native and actor provenance cannot be substituted"
        );
        drop(captured);
        drop(fresh);
        runtime.snapshots().release_snapshot(&old_handle).unwrap();
        runtime
            .snapshots()
            .release_snapshot(&changed_handle)
            .unwrap();
    });
}
