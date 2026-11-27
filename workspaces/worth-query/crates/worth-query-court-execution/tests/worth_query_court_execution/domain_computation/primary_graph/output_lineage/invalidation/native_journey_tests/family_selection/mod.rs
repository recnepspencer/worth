//! Public #429: a consumer's `current_output` selection is a verifiable
//! dependency. A read that selected `Missing` must not stay current once the
//! producer's first completion gives the family a head.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

/// What a `current_output` read that selected `Missing` retains for its
/// consumer row. The read records no keyed fact for `Missing`
/// (`current_output/resolution.rs`: only `record_unkeyed_read`, which
/// refusals use), so the row has nothing a completion can dirty.
fn missing_read_facts() -> Arc<[WorthQueryApplicationObservedFact]> {
    Arc::from([])
}

/// Lineage doors in a fixed order: a consumer row with a `Missing` read is
/// registered, then the producer's first completion registers its settlement
/// (`prepare_settlement_at_basis` → `record_completion`). Certifying the
/// consumer at the completion's snapshot must not pass Current.
#[test]
#[ignore = "held on public #429: a Missing current_output read records no fact, so the first completion cannot dirty the consumer"]
fn a_missing_read_is_invalidated_by_a_first_completion() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let status_ref = AccountStatus::at("AccountStatus");
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource::new_for_test(
            (*world.application.runtime_for_test()).authority_identity_for_test().as_u64(),
            (*world.application.installed_schema_for_test()).binding_identity().clone(),
            worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity),
            output_binding,
        )
    };
    let producer = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("StatusOutput")),
        coordinate,
        0,
    );
    let consumer = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("DownstreamOutput")),
        coordinate,
        0,
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_for_test(request_execution, |runtime| {
                // The real companion needs a selected cell before the read.
                write_field(request_execution, runtime, entity, status.clone(), "prime");
                let (read_handle, read) = snapshot(runtime);
                register(owner, consumer.clone(), missing_read_facts(), &read, OrdSet::new());
                assert!(
                    matches!(currentness(owner, &read, &consumer), SourceSettlementCurrentness::Clean),
                    "the consumer is current where it read Missing"
                );
                write_field(request_execution, runtime, entity, status.clone(), "closed");
                let (completed_handle, completed) = snapshot(runtime);
                let producer_facts: Arc<[_]> =
                    Arc::from([field_fact(request_execution, runtime, &completed_handle, entity, status)]);
                register(owner, producer.clone(), producer_facts, &completed, OrdSet::new());
                assert!(
                    !matches!(currentness(owner, &completed, &consumer), SourceSettlementCurrentness::Clean),
                    "the producer's first completion selects Unique, so the consumer's Missing read must not pass Current (public #429)"
                );
                for snapshot in [read_handle, completed_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}
