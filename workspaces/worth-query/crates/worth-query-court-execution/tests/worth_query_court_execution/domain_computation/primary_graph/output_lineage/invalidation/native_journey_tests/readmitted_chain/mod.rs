//! Public #358: a readmitted consumer chain is posted bottom-up at the head
//! readmission compared. An upstream row that a declared unavailable change
//! left requiring verification is re-established there, and the consumer is
//! Clean with no pending upstream; a dirty upstream refuses before the
//! consumer gets a row.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::request_recording::{
    account_witness, primed_output,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::reestablishment::ReadmittedNode;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_query_execution::facade::primary_graph::{
    readmitted_chains_on_this_thread_for_test, settlement_registrations_on_this_thread_for_test,
};

#[test]
fn readmitted_chain_after_declared_change_settles_clean() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, upstream) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let slot = |slot| {
        RecordedSettlementIdentity::retain_for_test(upstream.source_for_test(), upstream.coordinate_for_test(), slot)
    };
    let consumer = slot(1);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let label_ref = AccountLabel::at("AccountLabel");
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); handle.with_runtime(request_execution, |runtime| {
        let execution = &host_custody.borrowed_request_for_test();
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let head = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let facts: Arc<[WorthQueryApplicationObservedFact]> =
            Arc::from([field_fact(execution, runtime, snapshot, entity, label.clone())]);
        let node = |identity: &Arc<RecordedSettlementIdentity>, upstream: &[&Arc<RecordedSettlementIdentity>]| ReadmittedNode {
            identity: Arc::clone(identity),
            facts: RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts))
                .for_comparison()
                .unwrap(),
            witness: account_witness(execution, &world, runtime, snapshot, entity),
            upstream: upstream.iter().map(|identity| Arc::clone(identity)).collect(),
            historical: None,
        };
        let read = |identity: &Arc<RecordedSettlementIdentity>| currentness(owner, &head, identity);

        // 1. A Clean upstream, then a declared unavailable change at the head.
        register(owner, Arc::clone(&upstream), Arc::clone(&facts), &head, OrdSet::new());
        assert!(matches!(read(&upstream), SourceSettlementCurrentness::Clean));
        owner.apply_discontinuity_for_test(&head).expect("the head image takes the discontinuity");
        assert!(matches!(
            read(&upstream),
            SourceSettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::DeclaredChangeUnavailable
            )
        ));

        // 2. The chain is posted: the upstream is re-established, the
        // consumer registered over it, and both are Clean.
        let chain = [node(&upstream, &[]), node(&consumer, &[&upstream])];
        let readmitted = readmitted_chains_on_this_thread_for_test();
        assert!(matches!(
            owner.post_readmitted_chain(runtime, snapshot, &head, &chain),
            Ok(true)
        ));
        assert!(matches!(read(&upstream), SourceSettlementCurrentness::Clean));
        assert!(
            matches!(read(&consumer), SourceSettlementCurrentness::Clean),
            "a Clean consumer has no pending upstream"
        );
        assert_eq!(readmitted_chains_on_this_thread_for_test(), readmitted + 1);

        // 3. Posting the same chain again keeps both rows: nothing registers.
        let registrations = settlement_registrations_on_this_thread_for_test();
        assert!(matches!(
            owner.post_readmitted_chain(runtime, snapshot, &head, &chain),
            Ok(true)
        ));
        assert_eq!(
            settlement_registrations_on_this_thread_for_test(),
            registrations,
            "a Clean readmitted chain is kept, never registered again"
        );

        // 4. A dirty upstream (an in-window delivery) refuses before its
        // consumer gets a row.
        let (dirty, dirty_consumer) = (slot(2), slot(3));
        owner
            .register_settlement_for_test(SettlementRegistration::new_for_test(None, None, Arc::clone(&dirty), RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)), None, head.clone(), OrdSet::unit(0), None, OrdSet::new()))
            .expect("the dirty upstream registers");
        assert!(matches!(read(&dirty), SourceSettlementCurrentness::Dirty(_)));
        assert!(matches!(
            owner.post_readmitted_chain(runtime, snapshot, &head, &[node(&dirty, &[]), node(&dirty_consumer, &[&dirty])]),
            Ok(false)
        ));
        assert!(
            !owner
                .cell_for_read_for_test(&head)
                .unwrap()
                .unwrap()
                .read_image()
                .payload()
                .current
                .settlements_for_test()
                .contains_key(&dirty_consumer),
            "a refused chain leaves its consumer without a row"
        );
    }) }).expect("the fixture read owns its installed host call");
}
