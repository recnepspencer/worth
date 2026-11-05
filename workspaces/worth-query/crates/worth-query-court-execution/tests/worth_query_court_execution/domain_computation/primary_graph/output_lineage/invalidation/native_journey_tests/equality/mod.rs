//! Native actor consequence with real source snapshots and delivered touches.
//! The test-only relation mint exercises the actor, not producer authorization.
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::stable_publication::source_registration::StableEqualityConsequence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::SettlementCurrentness;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState;

#[test]
fn certified_alias_chain_discharge_is_exact_and_later_native_change_repends() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|host_phase| {
            world.selected_product(&host_phase.test_host_request_for_test())
        })
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("installed host call opens");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
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
        SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding)
    };
    let a_source = source(OutputBindingIdentity::declared_for_test("StatusOutput"));
    let a0 = RecordedSettlementIdentity::retain_for_test(&a_source, coordinate, 0);
    let a1 = RecordedSettlementIdentity::retain_for_test(&a_source, coordinate, 1);
    let a2 = RecordedSettlementIdentity::retain_for_test(&a_source, coordinate, 2);
    let b = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("DownstreamOutput")),
        coordinate,
        0,
    );
    let c = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("LateOutput")),
        coordinate,
        0,
    );
    let other = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("OtherUpstream")),
        coordinate,
        0,
    );

    handle.with_runtime_for_test(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, label.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let old_a_fact = field_fact(request_execution, runtime, &before_handle, entity, status.clone());
        register(owner, Arc::clone(&a0), Arc::from([old_a_fact.clone()]), &before, OrdSet::new());
        register(owner, Arc::clone(&other), Arc::from([field_fact(request_execution, runtime, &before_handle, entity, label.clone())]), &before, OrdSet::new());
        register(owner, Arc::clone(&b), Arc::from([]), &before, OrdSet::unit(Arc::clone(&a0)));
        register(owner, Arc::clone(&c), Arc::from([]), &before, [Arc::clone(&b), Arc::clone(&other)].into_iter().collect());

        write_field(request_execution, runtime, entity, status.clone(), "closed");
        write_field(request_execution, runtime, entity, label, "changed");
        let (changed_handle, changed) = snapshot(runtime);
        assert_pending(owner, &changed, &b, &a0);
        assert_pending(owner, &changed, &c, &b);
        assert_pending(owner, &changed, &c, &other);


        let historical_image = owner.cell_for_read_for_test(&changed).unwrap().unwrap().read_image();
        register_alias(request_execution, owner, runtime, &changed_handle, &changed, entity, status.clone(), &a0, &a1);
        assert!(matches!(currentness(owner, &changed, &b), SourceSettlementCurrentness::Clean));
        assert_pending(owner, &changed, &c, &b);
        assert_pending(owner, &changed, &c, &other);
        let prior = SnapshotAlignedMarkState::observe_image_for_test(&historical_image, &changed).unwrap();
        assert!(matches!(prior.currentness_for_test(&b), SettlementCurrentness::PendingUpstream(edges) if edges.contains(&a0)));
        assert_source_only_alias_verification(request_execution, owner, runtime, &changed_handle, &changed, &before, &a0, &old_a_fact);
        // Late registration consumes the already-certified clean successor,
        // rather than recreating a pending edge from the dirty predecessor.
        register(owner, Arc::clone(&b), Arc::from([]), &changed, OrdSet::unit(Arc::clone(&a0)));
        assert!(matches!(currentness(owner, &changed, &b), SourceSettlementCurrentness::Clean));

        write_field(request_execution, runtime, entity, status.clone(), "again");
        let (next_handle, next) = snapshot(runtime);
        assert_pending(owner, &next, &b, &a0);
        register_alias(request_execution, owner, runtime, &next_handle, &next, entity, status.clone(), &a1, &a2);
        assert!(matches!(currentness(owner, &next, &b), SourceSettlementCurrentness::Clean));
        assert_source_only_alias_verification(request_execution, owner, runtime, &next_handle, &next, &before, &a0, &old_a_fact);

        write_field(request_execution, runtime, entity, status.clone(), "changed-output");
        let (later_handle, later) = snapshot(runtime);
        assert_pending(owner, &later, &b, &a0);
        assert!(matches!(
            currentness(owner, &later, &a2),
            SourceSettlementCurrentness::Dirty(_)
        ));
        assert_eq!(
            ConsumedOutputEvidence::verify_candidate_with_admission_for_test(
request_execution,                &a0,
                &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::from([old_a_fact.clone()])).for_comparison().unwrap(),
                &[],
                None,
                &before,
                owner,
                runtime,
                &later_handle,
                &later,
            ),
            Ok(ConsumedOutputVerification::ChangedDirectFact(0)),
        );
        for handle in [before_handle, changed_handle, next_handle, later_handle] {
            runtime.snapshots().release_snapshot(&handle).unwrap();
        }
    });
}

#[test]
fn unchanged_own_evidence_waits_for_a_pending_equal_successor() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|host_phase| {
            world.selected_product(&host_phase.test_host_request_for_test())
        })
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("installed host call opens");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
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
        SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding)
    };
    let a_source = source(OutputBindingIdentity::declared_for_test("StatusOutput"));
    let a0 = RecordedSettlementIdentity::retain_for_test(&a_source, coordinate, 0);
    let a1 = RecordedSettlementIdentity::retain_for_test(&a_source, coordinate, 1);
    handle.with_runtime_for_test(request_execution, |runtime| {
        let (before_handle, before) = snapshot(runtime);
        let unchanged_fact = field_fact(request_execution, runtime, &before_handle, entity, label);
        register(owner, Arc::clone(&a0), Arc::from([unchanged_fact.clone()]), &before, OrdSet::new());
        write_field(request_execution, runtime, entity, status.clone(), "closed");
        let (alias_handle, alias) = snapshot(runtime);
        register_alias(request_execution, owner, runtime, &alias_handle, &alias, entity, status.clone(), &a0, &a1);
        // The successor's status changes; the predecessor's label never changes.
        write_field(request_execution, runtime, entity, status, "changed-output");
        let (later_handle, later) = snapshot(runtime);
        assert!(matches!(owner.consumed_output_currentness_for_test(&later, &a0).unwrap(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::ConsumedOutputCurrentness::PendingEqualSuccessor));
        assert_eq!(ConsumedOutputEvidence::verify_candidate_with_admission_for_test(
request_execution,            &a0,
            &RetainedSourceFacts::for_test_for_test(false, Arc::from([unchanged_fact])).for_comparison().unwrap(),
            &[], None, &before, owner, runtime, &later_handle, &later,
        ), Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop::PendingUpstream));
        for handle in [before_handle, alias_handle, later_handle] {
            runtime.snapshots().release_snapshot(&handle).unwrap();
        }
    });
}

pub fn register_alias(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    owner: &SourceInvalidationOwner,
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    selected: &PositionedRelationalSnapshot,
    entity: EntityId,
    locator: AspectFieldLocator,
    predecessor: &Arc<RecordedSettlementIdentity>,
    successor: &Arc<RecordedSettlementIdentity>,
) {
    let relation = StableEqualityConsequence::native_actor_fixture_for_test(
        Arc::clone(predecessor),
        Arc::clone(successor),
        selected,
    );
    let prepared = owner
        .prepare_current_stable_settlement_for_test(
            SettlementRegistration::new_for_test(
                None,
                None,
                Arc::clone(successor),
                RetainedSourceFacts::for_test_for_test(
                    false,
                    Arc::from([field_fact(execution, runtime, handle, entity, locator)]),
                ),
                None,
                selected.clone(),
                OrdSet::new(),
                None,
                OrdSet::new(),
            ),
            selected,
            relation,
        )
        .expect("the exact selected alias prepares");

    let cleanup = match prepared.install_for_test() {
        Ok(cleanup) => cleanup,
        Err(_) => panic!("the exact selected source image must install"),
    };
    drop(cleanup);
}

pub fn assert_pending(
    owner: &SourceInvalidationOwner,
    selected: &PositionedRelationalSnapshot,
    target: &Arc<RecordedSettlementIdentity>,
    predecessor: &Arc<RecordedSettlementIdentity>,
) {
    match currentness(owner, selected, target) {
        SourceSettlementCurrentness::PendingUpstream(edges) => assert!(edges.contains(predecessor)),
        _ => panic!("only the exact consumed predecessor can be pending"),
    }
}

pub fn assert_source_only_alias_verification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    owner: &SourceInvalidationOwner,
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    selected: &PositionedRelationalSnapshot,
    observed: &PositionedRelationalSnapshot,
    old_identity: &Arc<RecordedSettlementIdentity>,
    old_fact: &WorthQueryApplicationObservedFact,
) {
    let verify = || {
        ConsumedOutputEvidence::verify_candidate_with_admission_for_test(
            execution,
            old_identity,
            &RetainedSourceFacts::for_test_for_test(false, Arc::from([old_fact.clone()]))
                .for_comparison()
                .unwrap(),
            &[],
            None,
            observed,
            owner,
            runtime,
            handle,
            selected,
        )
    };
    assert_eq!(verify(), Ok(ConsumedOutputVerification::Current));
    let _oracle =
        worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest::arm_for_invalidation_resources_for_test(&(*owner.resources_for_test()));
    assert_eq!(
        verify(),
        Err(ConsumedOutputVerificationStop::Unavailable),
        "actor-only aliases cannot supply the missing performed output projection",
    );
}

#[path = "pending_resolution/mod.rs"]
pub mod pending_resolution;
