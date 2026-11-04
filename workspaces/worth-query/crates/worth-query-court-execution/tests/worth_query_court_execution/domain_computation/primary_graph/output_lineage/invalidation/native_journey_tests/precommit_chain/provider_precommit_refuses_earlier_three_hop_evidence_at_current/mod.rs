use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::precommit_chain::*;

#[test]
fn provider_precommit_refuses_earlier_three_hop_evidence_at_current_submission() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, product) = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let entity = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test();
            let (_, product, _) = selected.into_parts_for_test();
            (entity, product)
        })
        .expect("initial source selection borrows its installed phase");
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding)
    };
    let a = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("StatusOutput")),
        coordinate,
        0,
    );
    let b = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("DownstreamOutput")),
        coordinate,
        0,
    );
    publish_field(request_execution, &world, entity, label.clone(), "prime");
    let (before_handle, before, a_facts) = handle.with_runtime(request_execution, |runtime| {
        let (selected, basis) = snapshot(runtime);
        let facts: Arc<[_]> = Arc::from([field_fact(
            request_execution,
            runtime,
            &selected,
            entity,
            status.clone(),
        )]);
        (selected, basis, facts)
    });
    register(
        owner,
        Arc::clone(&a),
        Arc::clone(&a_facts),
        &before,
        OrdSet::new(),
    );
    register(
        owner,
        Arc::clone(&b),
        Arc::from([]),
        &before,
        OrdSet::unit(Arc::clone(&a)),
    );
    let a_evidence = ConsumedOutputEvidence::retained_for_test(
        Arc::clone(&a),
        a_facts,
        Vec::new(),
        None,
        Arc::new(before.clone()),
    );
    let b_evidence = ConsumedOutputEvidence::retained_for_test(
        Arc::clone(&b),
        Arc::from([]),
        vec![a_evidence],
        None,
        Arc::new(before),
    );

    // World publication advances both the native and product selected roots.
    // The unrelated field leaves the actual consumed source fact current.
    publish_field(request_execution, &world, entity, label, "unrelated-label");
    let (unrelated_handle, unrelated) = handle.with_runtime(request_execution, snapshot);

    assert_eq!(
        handle.with_runtime(request_execution, |runtime| {
            ConsumedOutputEvidence::verify_many_with_admission_for_test(
                request_execution,
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &unrelated_handle,
                &unrelated,
            )
        }),
        Ok(ConsumedOutputVerification::Current),
    );

    // C has already consumed B. Changing A leaves B pending at the new
    // product-selected source, before C's provider commit can prepare effects.
    publish_field(request_execution, &world, entity, status, "closed");
    let (after_handle, after) = handle.with_runtime(request_execution, snapshot);

    assert_eq!(
        handle.with_runtime(request_execution, |runtime| {
            ConsumedOutputEvidence::verify_many_with_admission_for_test(
                request_execution,
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &after_handle,
                &after,
            )
        }),
        Err(ConsumedOutputVerificationStop::PendingUpstream),
    );
    // A discontinuous B row has no direct facts. Its shared A evidence must
    // still catch the changed source fact after full verification is required.
    owner
        .register_settlement_for_test(SettlementRegistration::new_for_test(
            None,
            None,
            Arc::clone(&b),
            RetainedSourceFacts::for_test_for_test(false, Arc::from([])),
            None,
            after.clone(),
            OrdSet::new(),
            Some(FullVerificationReason::DeclaredChangeUnavailable),
            OrdSet::unit(Arc::clone(&a)),
        ))
        .expect("discontinuous B row is retained at the selected source");
    assert!(matches!(
        owner.currentness_for_test(&after, &b),
        Ok(SourceSettlementCurrentness::FullVerificationRequired(
            FullVerificationReason::DeclaredChangeUnavailable
        ))
    ));

    assert_eq!(
        handle.with_runtime(request_execution, |runtime| {
            ConsumedOutputEvidence::verify_many_with_admission_for_test(
                request_execution,
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &after_handle,
                &after,
            )
        }),
        Ok(ConsumedOutputVerification::ChangedUpstream),
    );
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        for selected in [before_handle, unrelated_handle, after_handle] {
            runtime.snapshots().release_snapshot(&selected).unwrap();
        }
    });

    // The real provider commit entry must consume the earlier B evidence
    // before preparing any effect. C is submitted at the current native and
    // Product basis, so a historical-basis CAS cannot explain its refusal.
    let admitted_program = |replacement: &str| {
        world
            .application
            .with_host_advancement(|phase| {
                let request = live_scope();
                let external = world.authenticate("alice", Duration::from_secs(60), &request);
                let selected = world.selected_product(&phase.test_host_request_for_test());
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &world.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let account = selected
                    .resolve_entity(
                        &phase,
                        AccountStatus::reference(),
                        "closed".to_owned(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = world
                    .application
                    .installed_schema()
                    .installed_operation(TouchAccountOperation::reference())
                    .unwrap();
                let admission = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &account,
                        &operation,
                        Default::default(),
                        &request,
                    )
                    .unwrap();
                let (_, projection, _) = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, account| {
                            reader
                                .require_decision_field(account, AccountStatus::reference())
                                .unwrap();
                        },
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let reads = world
                    .application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                let mut effects = reads
                    .complete_projected_dependencies(
                        &phase,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .begin_effect_program();
                let account = effects.existing_entity(&account).unwrap();
                effects
                    .write_field(&account, AccountStatus::reference(), replacement.to_owned())
                    .unwrap();
                effects.finish().unwrap()
            })
            .expect("program reads borrow one installed phase")
    };
    let stale = admitted_program("must-not-publish").with_consumed_output_for_test(b_evidence);
    let product_before = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("product selection borrows its installed phase")
        .product()
        .observation_for_test()
        .clone();
    let (head_before_handle, head_before) = handle.with_runtime(request_execution, snapshot);
    let denied = world.application.compare_and_commit_application(
        stale,
        WorthQueryApplicationIdempotencyBinding::new_for_test([211; 32], [212; 32]),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = denied else {
        panic!("the provider must refuse changed upstream evidence before effects: {denied:?}");
    };
    assert_eq!(
        denial.detail(),
        Some("consumed output changed before application publication"),
        "precommit refusal kind {:?}, stage {:?}",
        denial.kind(),
        denial.stage(),
    );
    let (head_after_handle, head_after) = handle.with_runtime(request_execution, snapshot);
    assert_eq!(
        head_after, head_before,
        "the denied C commit cannot move native truth"
    );
    assert_eq!(
        world
            .application
            .with_host_advancement(
                |phase| world.selected_product(&phase.test_host_request_for_test())
            )
            .expect("product selection borrows its installed phase")
            .product()
            .observation_for_test(),
        &product_before,
        "the denied C commit cannot move Product truth"
    );
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&head_before_handle)
            .unwrap();
        runtime
            .snapshots()
            .release_snapshot(&head_after_handle)
            .unwrap();
    });
    assert!(matches!(
        world.application.compare_and_commit_application(
            admitted_program("fresh-valid-commit"),
            WorthQueryApplicationIdempotencyBinding::new_for_test([213; 32], [214; 32]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
