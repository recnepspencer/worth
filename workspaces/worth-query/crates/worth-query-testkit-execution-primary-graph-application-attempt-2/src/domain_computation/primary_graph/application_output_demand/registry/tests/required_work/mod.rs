use std::sync::Arc;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::queue::RequiredWorkPop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::queue::RequiredWorkQueue;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWork;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWorkKind;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture;
pub trait RequiredWorkWorthQueryOutputDemandRegistryForTest: Sized {
    fn fixture_requeue_admitted_work(
        &self,
        interest: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
    );
    fn fixture_admitted_work_membership(
        &self, occurrence: worth_runtime_world::facade::ProductBranchIncarnation) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    );
    fn fixture_scope_work_membership(
        &self, occurrence: worth_runtime_world::facade::ProductBranchIncarnation, root: worth_relational::facade::identity::EntityId) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    );
    fn fixture_work_membership_at(
        &self, occurrence: worth_runtime_world::facade::ProductBranchIncarnation, root: worth_relational::facade::identity::EntityId, output: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    );
}
impl RequiredWorkWorthQueryOutputDemandRegistryForTest for worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry {
fn fixture_requeue_admitted_work(
        &self,
        interest: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
    ) {
        let member = (*self.state_for_test()).lock().unwrap().records[&(*interest.key_for_test())]
            .work_membership
            .as_ref()
            .cloned()
            .expect("admitted required record retains its token");
        member.set_required(true);
    }

fn fixture_admitted_work_membership(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        self.fixture_work_membership_at(occurrence, crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::root(1), key("native-work-member", 1, 1))
    }

fn fixture_scope_work_membership(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        root: worth_relational::facade::identity::EntityId,
    ) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        let output = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey::new_for_test("registry-output-family", "native-work-member".to_owned(), worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability::new("registry-fixture", worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch::new_for_test(
                [1; 32], [2; 32], root, occurrence, 1, [0; 32]));
        self.fixture_work_membership_at(occurrence, root, output)
    }

fn fixture_work_membership_at(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        root: worth_relational::facade::identity::EntityId,
        output: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey,
    ) -> (
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest,
        Arc<RequiredWorkMembership>,
    ) {
        let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root);
        let interest = self
            .admit_for_test(
                output.clone(),
                None,
                scope,
                occurrence,
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Required,
                None,
                None,
            )
            .expect("real required-interest admission creates its work token")
            .0;
        let member = (*self.state_for_test()).lock().unwrap().records[&output]
            .work_membership
            .as_ref()
            .cloned()
            .expect("required record retains its work token");
        (interest, member)
    }
}
pub fn selected(queue: &RequiredWorkQueue) -> SelectedRequiredWork {
    match queue.pop() {
        RequiredWorkPop::Claimed(claim) => claim
            .select_without_advance()
            .expect("a live claim selects its current cause"),
        RequiredWorkPop::Empty | RequiredWorkPop::Stale => panic!("expected selected work"),
    }
}
