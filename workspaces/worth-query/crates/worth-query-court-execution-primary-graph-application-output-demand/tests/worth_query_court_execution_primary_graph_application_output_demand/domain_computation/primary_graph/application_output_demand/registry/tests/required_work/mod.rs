use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::acknowledgement_basis::AcknowledgementBasisCurrentProofForTest;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::selected;

use std::sync::Arc;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::queue::RequiredWorkPop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::queue::RequiredWorkQueue;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWork;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWorkKind;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture;
#[test]
fn activation_and_newer_hint_survive_an_older_selected_ack() {
    let (_lineage, [earlier, later]) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new());
    let member = Arc::new(RequiredWorkMembership::new_for_test(
        Arc::new(key("selected-output", 1, 1)),
        &queue,
    ));
    // Preparation can mark an inactive row before its required interest opens.
    drop(member.mark_local_required_for_test(Arc::clone(&earlier)));
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
    member.set_required_for_test(true);
    let first = selected(&queue);
    assert!(matches!(
        first.kind_for_test(),
        SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &earlier)
    ));
    // A newer cutover must survive the older worker's acknowledgement.
    drop(member.mark_local_required_for_test(Arc::clone(&later)));
    assert!(first.acknowledge_fixture().is_none());
    let selected = selected(&queue);
    assert!(matches!(
        selected.kind_for_test(),
        SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &later)
    ));
    assert!(selected.acknowledge_fixture().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}
#[test]
fn new_required_record_selects_unresolved_work_without_a_settlement_identity() {
    let queue = Arc::new(RequiredWorkQueue::new());
    let member = Arc::new(RequiredWorkMembership::new_for_test(
        Arc::new(key("unsettled-output", 1, 1)),
        &queue,
    ));
    member.set_required_for_test(true);
    let selected = selected(&queue);
    assert!(matches!(
        selected.kind_for_test(),
        SelectedRequiredWorkKind::UnresolvedInitial
    ));
    assert!(selected.acknowledge_fixture().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}
#[test]
fn duplicate_selected_version_cannot_acknowledge_the_next_cause() {
    let (_lineage, [settlement, _]) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new());
    let member = Arc::new(RequiredWorkMembership::new_for_test(
        Arc::new(key("selected-output", 1, 1)),
        &queue,
    ));
    member.set_required_for_test(true); // Keeps an independent unresolved cause.
    drop(member.mark_local_required_for_test(Arc::clone(&settlement)));
    let first = selected(&queue);
    member.set_required_for_test(true); // Requeue without publishing a newer hint.
    let second = selected(&queue);
    assert!(matches!(
        first.kind_for_test(),
        SelectedRequiredWorkKind::Local { .. }
    ));
    assert!(matches!(
        second.kind_for_test(),
        SelectedRequiredWorkKind::Local { .. }
    ));
    assert!(first.acknowledge_fixture().is_some());
    assert!(second.acknowledge_fixture().is_none());
    let remaining = selected(&queue);
    assert!(matches!(
        remaining.kind_for_test(),
        SelectedRequiredWorkKind::UnresolvedInitial
    ));
    assert!(remaining.acknowledge_fixture().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}
#[test]
fn requeued_last_cause_becomes_one_stale_pop() {
    let queue = Arc::new(RequiredWorkQueue::new());
    let member = Arc::new(RequiredWorkMembership::new_for_test(
        Arc::new(key("unsettled-output", 1, 1)),
        &queue,
    ));
    member.set_required_for_test(true);
    let first = selected(&queue);
    member.set_required_for_test(true);
    assert!(first.acknowledge_fixture().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Stale));
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}
#[test]
fn a_claim_selects_newer_causes_and_current_acknowledges_the_row_once() {
    let (_lineage, [earlier, later]) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new());
    let member = Arc::new(RequiredWorkMembership::new_for_test(
        Arc::new(key("claimed-output", 1, 1)),
        &queue,
    ));
    member.set_required_for_test(true);
    drop(member.mark_local_required_for_test(earlier));
    let RequiredWorkPop::Claimed(mut claim) = queue.pop() else {
        panic!("a marked row is claimed")
    };
    drop(member.mark_local_required_for_test(Arc::clone(&later)));
    assert!(
        matches!(queue.pop(), RequiredWorkPop::Empty),
        "a live claim prevents another visit in this advance"
    );
    let selected = claim.select().unwrap();
    assert!(
        matches!(selected.kind_for_test(), SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &later)),
        "inspection selects the live cause, not the popped one"
    );
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof::with_current_fixture_basis(&member, |proof| {
        assert!(selected.acknowledge_current_admitted_for_test(proof).unwrap().is_some());
    });
    drop(claim);
    assert!(
        matches!(queue.pop(), RequiredWorkPop::Empty),
        "one Current proof covers this row's local and initial causes"
    );
}
#[test]
fn discontinuity_selects_the_first_declared_family_member() {
    for families in worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key::family_declarations() {
        assert_ne!(families[0].0, families[1].0);
        let registry = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry::default();
        let occurrence = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence();
        let seed = key("same-producer", 1, 1);
        let outputs = families.map(|(type_id, identity)| {
            (
                type_id,
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey::new_for_test(
                    identity,
                    seed.producer_identity_for_test().to_owned(),
                    seed.applicability(),
                    seed.source_epoch_for_test().clone(),
                ),
            )
        });
        let mut interests = Vec::new();
        for (_, output) in outputs.iter().rev() {
            interests.push(
                registry
                    .fixture_work_membership_at(occurrence, worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::root(1), output.clone())
                    .0,
            );
        }
        let queue = (*(*registry
            .state_for_test())
            .lock()
            .unwrap()
            .required_work_queue_for_test())
            .clone()
            .unwrap();
        for _ in 0..2 {
            assert!(selected(&queue).acknowledge_fixture().is_some());
        }
        (*registry
            .state_for_test())
            .lock()
            .unwrap()
            .fixture_page_required_discontinuity_for_test(
                occurrence,
                &worth_relational::facade::history::BranchId("main".to_owned()),
                worth_relational::facade::history::CommitId(1),
            )
            .unwrap();
        let first = selected(&queue);
        let selected = outputs.iter().find(|(_, key)| key == first.key_for_test()).unwrap();
        assert_eq!(selected, &outputs[0]);
        assert_ne!(selected.0, outputs[1].0);
    }
}
