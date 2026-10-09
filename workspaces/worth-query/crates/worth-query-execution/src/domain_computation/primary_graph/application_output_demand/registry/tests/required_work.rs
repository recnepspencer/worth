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
        let output = super::WorthQueryOutputDemandKey::new(std::any::TypeId::of::<super::support::RegistryOutputFamily>(), "native-work-member".to_owned(), crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability::new("registry-fixture", crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
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
        RequiredWorkPop::Selected(selected) => selected,
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
