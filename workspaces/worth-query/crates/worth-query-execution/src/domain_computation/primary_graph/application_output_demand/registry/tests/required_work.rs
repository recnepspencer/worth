use std::sync::Arc;

use super::{key, test_record_capacity};
use crate::domain_computation::primary_graph::application_output_demand::registry::required_work::{
    RequiredWorkMembership, RequiredWorkPop, RequiredWorkQueue, SelectedRequiredWork,
    SelectedRequiredWorkKind,
};
use crate::domain_computation::primary_graph::output_lineage::registry_fixture;

impl super::WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn fixture_requeue_admitted_work(
        &self,
        interest: &super::WorthQueryOutputDemandInterest,
    ) {
        let member = self.state.lock().unwrap().records[&interest.key]
            .work_membership
            .as_ref()
            .cloned()
            .expect("admitted required record retains its token");
        member.set_required(true);
    }

    pub(in crate::domain_computation::primary_graph) fn fixture_admitted_work_membership(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> (
        super::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        self.fixture_work_membership_at(occurrence, super::root(1), key("native-work-member", 1, 1))
    }

    pub(in crate::domain_computation::primary_graph) fn fixture_scope_work_membership(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        root: worth_relational::facade::identity::EntityId,
    ) -> (
        super::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        let output = super::WorthQueryOutputDemandKey::new("registry-output-family", "native-work-member".to_owned(), crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability::new("registry-fixture", crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
            crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch::new(
                [1; 32], [2; 32], root, occurrence, 1, [0; 32]));
        self.fixture_work_membership_at(occurrence, root, output)
    }

    fn fixture_work_membership_at(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        root: worth_relational::facade::identity::EntityId,
        output: super::WorthQueryOutputDemandKey,
    ) -> (
        super::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root);
        let interest = self
            .admit(
                output.clone(),
                None,
                scope,
                occurrence,
                super::super::DemandAdmissionKind::Required,
                None,
                None,
                &mut super::record_admission(),
            )
            .expect("real required-interest admission creates its work token");
        let member = self.state.lock().unwrap().records[&output]
            .work_membership
            .as_ref()
            .cloned()
            .expect("required record retains its work token");
        (interest, member)
    }
}

fn selected(queue: &RequiredWorkQueue) -> SelectedRequiredWork {
    match queue.pop() {
        RequiredWorkPop::Claimed(claim) => claim
            .select_without_advance()
            .expect("a live claim selects its current cause"),
        RequiredWorkPop::Empty | RequiredWorkPop::Stale => panic!("expected selected work"),
    }
}

#[test]
fn activation_and_newer_hint_survive_an_older_selected_ack() {
    let (_lineage, [earlier, later]) = registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new(Arc::new(test_record_capacity())));
    let member = Arc::new(RequiredWorkMembership::new(
        Arc::new(key("selected-output", 1, 1)),
        &queue,
        test_record_capacity(),
    ));
    // Preparation can mark an inactive row before its required interest opens.
    drop(member.mark_local_required(Arc::clone(&earlier)));
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
    member.set_required(true);
    let first = selected(&queue);
    assert!(matches!(
        first.kind(),
        SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &earlier)
    ));
    // A newer cutover must survive the older worker's acknowledgement.
    drop(member.mark_local_required(Arc::clone(&later)));
    assert!(first.acknowledge().is_none());
    let selected = selected(&queue);
    assert!(matches!(
        selected.kind(),
        SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &later)
    ));
    assert!(selected.acknowledge().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}

#[test]
fn new_required_record_selects_unresolved_work_without_a_settlement_identity() {
    let queue = Arc::new(RequiredWorkQueue::new(Arc::new(test_record_capacity())));
    let member = Arc::new(RequiredWorkMembership::new(
        Arc::new(key("unsettled-output", 1, 1)),
        &queue,
        test_record_capacity(),
    ));
    member.set_required(true);
    let selected = selected(&queue);
    assert!(matches!(
        selected.kind(),
        SelectedRequiredWorkKind::UnresolvedInitial
    ));
    assert!(selected.acknowledge().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}

#[test]
fn duplicate_selected_version_cannot_acknowledge_the_next_cause() {
    let (_lineage, [settlement, _]) = registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new(Arc::new(test_record_capacity())));
    let member = Arc::new(RequiredWorkMembership::new(
        Arc::new(key("selected-output", 1, 1)),
        &queue,
        test_record_capacity(),
    ));
    member.set_required(true); // Keeps an independent unresolved cause.
    drop(member.mark_local_required(Arc::clone(&settlement)));
    let first = selected(&queue);
    member.set_required(true); // Requeue without publishing a newer hint.
    let second = selected(&queue);
    assert!(matches!(
        first.kind(),
        SelectedRequiredWorkKind::Local { .. }
    ));
    assert!(matches!(
        second.kind(),
        SelectedRequiredWorkKind::Local { .. }
    ));
    assert!(first.acknowledge().is_some());
    assert!(second.acknowledge().is_none());
    let remaining = selected(&queue);
    assert!(matches!(
        remaining.kind(),
        SelectedRequiredWorkKind::UnresolvedInitial
    ));
    assert!(remaining.acknowledge().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}

#[test]
fn requeued_last_cause_becomes_one_stale_pop() {
    let queue = Arc::new(RequiredWorkQueue::new(Arc::new(test_record_capacity())));
    let member = Arc::new(RequiredWorkMembership::new(
        Arc::new(key("unsettled-output", 1, 1)),
        &queue,
        test_record_capacity(),
    ));
    member.set_required(true);
    let first = selected(&queue);
    member.set_required(true);
    assert!(first.acknowledge().is_some());
    assert!(matches!(queue.pop(), RequiredWorkPop::Stale));
    assert!(matches!(queue.pop(), RequiredWorkPop::Empty));
}

#[test]
fn a_claim_selects_newer_causes_and_current_acknowledges_the_row_once() {
    let (_lineage, [earlier, later]) = registry_fixture::recorded_settlement_pair();
    let queue = Arc::new(RequiredWorkQueue::new(Arc::new(test_record_capacity())));
    let member = Arc::new(RequiredWorkMembership::new(
        Arc::new(key("claimed-output", 1, 1)),
        &queue,
        test_record_capacity(),
    ));
    member.set_required(true);
    drop(member.mark_local_required(earlier));
    let RequiredWorkPop::Claimed(mut claim) = queue.pop() else {
        panic!("a marked row is claimed")
    };
    drop(member.mark_local_required(Arc::clone(&later)));
    assert!(
        matches!(queue.pop(), RequiredWorkPop::Empty),
        "a live claim prevents another visit in this advance"
    );
    let selected = claim.select().unwrap();
    assert!(
        matches!(selected.kind(), SelectedRequiredWorkKind::Local { settlement } if Arc::ptr_eq(settlement, &later)),
        "inspection selects the live cause, not the popped one"
    );
    assert!(selected
        .acknowledge_current_admitted(&mut super::record_admission())
        .unwrap()
        .is_some());
    drop(claim);
    assert!(
        matches!(queue.pop(), RequiredWorkPop::Empty),
        "one Current proof covers this row's local and initial causes"
    );
}

#[test]
fn discontinuity_selects_the_first_declared_family_member() {
    for families in super::demand_key::family_declarations() {
        assert_ne!(families[0].0, families[1].0);
        let registry = super::WorthQueryOutputDemandRegistry::default();
        let occurrence = super::occurrence();
        let seed = key("same-producer", 1, 1);
        let outputs = families.map(|(type_id, identity)| {
            (
                type_id,
                super::WorthQueryOutputDemandKey::new(
                    identity,
                    seed.producer_identity().to_owned(),
                    seed.applicability(),
                    seed.source_epoch().clone(),
                ),
            )
        });
        let mut interests = Vec::new();
        for (_, output) in outputs.iter().rev() {
            interests.push(
                registry
                    .fixture_work_membership_at(occurrence, super::root(1), output.clone())
                    .0,
            );
        }
        let queue = registry
            .state
            .lock()
            .unwrap()
            .required_work_queue
            .clone()
            .unwrap();
        for _ in 0..2 {
            assert!(selected(&queue).acknowledge().is_some());
        }
        registry
            .state
            .lock()
            .unwrap()
            .fixture_page_required_discontinuity(
                occurrence,
                &worth_relational::facade::history::BranchId("main".to_owned()),
                worth_relational::facade::history::CommitId(1),
                &mut super::record_admission(),
            )
            .unwrap();
        let first = selected(&queue);
        let selected = outputs.iter().find(|(_, key)| key == first.key()).unwrap();
        assert_eq!(selected, &outputs[0]);
        assert_ne!(selected.0, outputs[1].0);
    }
}
