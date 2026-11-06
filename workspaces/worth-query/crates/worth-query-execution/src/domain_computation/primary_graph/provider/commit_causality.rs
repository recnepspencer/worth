use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn observe_application_commit_causality(
        &self,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveSubscription {
        self.live_delivery
            .open(product.observation())
            .expect("the test observer has subscription capacity")
    }

    pub(crate) fn set_application_commit_causality_limits(&self, batch_capacity: usize) {
        self.live_delivery.set_limits(batch_capacity);
    }

    pub(crate) fn set_application_commit_causality_reservation_hook(
        &self,
        hook: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    ) {
        self.live_delivery.set_after_reservation_hook(hook);
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_application_commit_causality(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial,
    >{
        self.live_delivery.reserve(execution, product)
    }

    pub(in crate::domain_computation::primary_graph) fn publish_application_commit_causality(
        &self,
        reservation: crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
        publication: crate::domain_computation::primary_graph::vocabulary::application_attempt::committed_publication::WorthQueryCommittedProductPublication,
        emissions: WorthQueryAdmittedApplicationEmissionBatch,
        predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PreparedPredicatePublication>,
    ) -> usize {
        self.live_delivery
            .publish(reservation, publication, emissions, predicate)
    }

    pub(in crate::domain_computation::primary_graph) fn committed_application_emissions(
        &self,
        publication: &crate::domain_computation::primary_graph::vocabulary::application_attempt::committed_publication::WorthQueryCommittedProductPublication,
    ) -> Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission>
{
        self.live_delivery.emissions(publication)
    }

    pub(crate) fn published_application_commit_count(&self) -> usize {
        self.live_delivery.published_commit_count()
    }

    pub(crate) fn active_application_commit_causality_partitions(&self) -> usize {
        self.live_delivery.active_partition_count()
    }
}

mod court_doors_for_test;
