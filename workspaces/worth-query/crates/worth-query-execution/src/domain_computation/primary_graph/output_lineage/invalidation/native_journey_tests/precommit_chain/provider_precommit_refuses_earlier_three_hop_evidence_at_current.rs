//! provider precommit refuses earlier three hop evidence at current submission.
use super::*;
#[path = "provider_precommit_refuses_earlier_three_hop_evidence_at_current/application_program.rs"]
mod application_program;
#[path = "provider_precommit_refuses_earlier_three_hop_evidence_at_current/field_locators.rs"]
mod field_locators;

#[test]
fn provider_precommit_refuses_earlier_three_hop_evidence_at_current_submission() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let (status, label) = field_locators::status_and_label(&world);
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let a =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<StatusOutput>()), coordinate, 0);
    let b = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            publish_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                label.clone(),
                "prime",
            )
        })
        .expect("the fixture read owns its installed host call");
    let (before_handle, before, a_facts) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            handle.with_runtime(|runtime| {
                let (selected, basis) = snapshot(runtime);
                let facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &selected,
                    entity,
                    status.clone(),
                )]);
                (selected, basis, facts)
            })
        })
        .expect("the fixture read owns its installed host call");
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
        owner,
        Arc::clone(&a),
        a_facts,
        Vec::new(),
        None,
        Arc::new(before.clone()),
    );
    let b_evidence = ConsumedOutputEvidence::retained_for_test(
        owner,
        Arc::clone(&b),
        Arc::from([]),
        vec![a_evidence],
        None,
        Arc::new(before),
    );
    request_custody::assert_backing_paid_by_the_request(owner, std::slice::from_ref(&b_evidence));

    // World publication advances both the native and product selected roots.
    // The unrelated field leaves the actual consumed source fact current.
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            publish_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                label,
                "unrelated-label",
            )
        })
        .expect("the fixture read owns its installed host call");
    let (unrelated_handle, unrelated) = handle.with_runtime(snapshot);
    let mut work = owner.edit_admission();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                handle.with_runtime(
                    |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                        &host_custody.borrowed_request_for_test(),
                        std::slice::from_ref(&b_evidence),
                        owner,
                        runtime,
                        &unrelated_handle,
                        &unrelated,
                        &mut work,
                    )
                ),
                Ok(ConsumedOutputVerification::Current),
            )
        })
        .expect("the fixture read owns its installed host call");

    // Two closures on one admitted meter must consume two real sets of Work
    // and scratch. A fresh allowance per closure would incorrectly pass the
    // second call under either one-short budget.
    let verify_unrelated = |admission: &mut InvalidationEditAdmission| {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                handle.with_runtime(|runtime| {
                    ConsumedOutputEvidence::verify_many_with_admission(
                        &host_custody.borrowed_request_for_test(),
                        std::slice::from_ref(&b_evidence),
                        owner,
                        runtime,
                        &unrelated_handle,
                        &unrelated,
                        admission,
                    )
                })
            })
            .expect("the fixture invocation owns its installed host call")
    };
    request_custody::assert_current_closures_share_custody(owner, verify_unrelated);

    // C has already consumed B. Changing A leaves B pending at the new
    // product-selected source, before C's provider commit can prepare effects.
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            publish_field(
                &host_custody.borrowed_request_for_test(),
                &world,
                entity,
                status,
                "closed",
            )
        })
        .expect("the fixture read owns its installed host call");
    let (after_handle, after) = handle.with_runtime(snapshot);
    let mut work = owner.edit_admission();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                handle.with_runtime(
                    |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                        &host_custody.borrowed_request_for_test(),
                        std::slice::from_ref(&b_evidence),
                        owner,
                        runtime,
                        &after_handle,
                        &after,
                        &mut work,
                    )
                ),
                Err(ConsumedOutputVerificationStop::PendingUpstream),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        work.charged_work() > 0,
        "the wrapper debits the work spent before denial"
    );

    // A discontinuous B row has no direct facts. Its shared A evidence must
    // still catch the changed source fact after full verification is required.
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(&b),
                facts: RetainedSourceFacts::for_test(false, Arc::from([])),
                output_facts: None,
                read_basis: after.clone(),
                stale_at_read_basis: OrdSet::new(),
                requirement: Some(FullVerificationReason::DeclaredChangeUnavailable),
                upstream: OrdSet::unit(Arc::clone(&a)),
            },
            &mut owner.edit_admission(),
        )
        .expect("discontinuous B row is retained at the selected source");
    assert!(matches!(
        owner.currentness(&after, &b, &mut owner.edit_admission()),
        Ok(SourceSettlementCurrentness::FullVerificationRequired(
            FullVerificationReason::DeclaredChangeUnavailable
        ))
    ));
    let mut work = owner.edit_admission();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                handle.with_runtime(
                    |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                        &host_custody.borrowed_request_for_test(),
                        std::slice::from_ref(&b_evidence),
                        owner,
                        runtime,
                        &after_handle,
                        &after,
                        &mut work,
                    )
                ),
                Ok(ConsumedOutputVerification::ChangedUpstream),
            )
        })
        .expect("the fixture read owns its installed host call");
    handle.with_runtime_mut(|runtime| {
        for selected in [before_handle, unrelated_handle, after_handle] {
            runtime.snapshots().release_snapshot(&selected).unwrap();
        }
    });

    // The real provider commit entry must consume the earlier B evidence
    // before preparing any effect. C is submitted at the current native and
    // Product basis, so a historical-basis CAS cannot explain its refusal.
    let admitted_program = |replacement: &str| application_program::admit(&world, replacement);
    let stale = admitted_program("must-not-publish").with_consumed_output_for_test(b_evidence);
    let product_before = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .observation()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let (head_before_handle, head_before) = handle.with_runtime(snapshot);
    let denied = world.application.compare_and_commit_application(
        stale,
        WorthQueryApplicationIdempotencyBinding::new([211; 32], [212; 32]),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
    let (head_after_handle, head_after) = handle.with_runtime(snapshot);
    assert_eq!(
        head_after, head_before,
        "the denied C commit cannot move native truth"
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .observation(),
                &product_before,
                "the denied C commit cannot move Product truth"
            )
        })
        .expect("the fixture read owns its installed host call");
    handle.with_runtime_mut(|runtime| {
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
            WorthQueryApplicationIdempotencyBinding::new([213; 32], [214; 32]),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
