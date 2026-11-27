//! Court 10, CE half (public #429, condition 3): readmission keeps a row that
//! marks cannot certify (`UnsupportedFact`) only when it compared that row in
//! full. A row outside the compared extent is refused, and its consumer gets
//! no row.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::request_recording::{
    account_witness, primed_output,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::reestablishment::ReadmittedNode;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_relational::facade::identity::KindId;

#[test]
fn readmission_refuses_an_unsupported_row_it_did_not_compare() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (entity, upstream) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let consumer = RecordedSettlementIdentity::retain_for_test(upstream.source_for_test(), upstream.coordinate_for_test(), 1);
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); handle.with_runtime(request_execution, |runtime| {
        let execution = &host_custody.borrowed_request_for_test();
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let head = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        // A workflow-definition read has no posting, so marks never certify
        // the row that holds it: it is `UnsupportedFact` at every head.
        let facts: Arc<[WorthQueryApplicationObservedFact]> =
            Arc::from([WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
                relation_kind: KindId::new(9),
                lineage: entity,
                expected_definition: entity,
                maximum_work_units: 1,
            }]);
        let node = |identity: &Arc<RecordedSettlementIdentity>, upstream: &[&Arc<RecordedSettlementIdentity>], fully_compared| ReadmittedNode {
            identity: Arc::clone(identity),
            facts: RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
            witness: account_witness(execution, &world, runtime, snapshot, entity),
            upstream: upstream.iter().map(|identity| Arc::clone(identity)).collect(),
            historical: None,
            fully_compared,
        };
        register(owner, Arc::clone(&upstream), Arc::clone(&facts), &head, OrdSet::new());
        assert!(matches!(
            currentness(owner, &head, &upstream),
            SourceSettlementCurrentness::FullVerificationRequired(FullVerificationReason::UnsupportedFact)
        ));
        let consumer_row = || {
            owner
                .cell_for_read_for_test(&head)
                .unwrap()
                .unwrap()
                .read_image()
                .payload()
                .current
                .settlements_for_test()
                .contains_key(&consumer)
        };

        // Outside the compared extent: refused, and the consumer gets no row.
        assert!(matches!(
            owner.post_readmitted_chain(runtime, snapshot, &head, &[node(&upstream, &[], false), node(&consumer, &[&upstream], false)]),
            Ok(false)
        ));
        assert!(!consumer_row(), "a refused chain leaves its consumer without a row");

        // Compared in full: kept as it is, and the consumer is posted over it.
        assert!(matches!(
            owner.post_readmitted_chain(runtime, snapshot, &head, &[node(&upstream, &[], true), node(&consumer, &[&upstream], true)]),
            Ok(true)
        ));
        assert!(consumer_row(), "an accepted chain posts its consumer");
        assert!(matches!(
            currentness(owner, &head, &upstream),
            SourceSettlementCurrentness::FullVerificationRequired(FullVerificationReason::UnsupportedFact)
        ));
    }) }).expect("the fixture read owns its installed host call");
}
