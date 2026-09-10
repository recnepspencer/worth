use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch;

impl WorthQueryPrimaryGraphProvider {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn observe_application_commit_causality(
        &self,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveSubscription {
        self.live_delivery.open(product.observation())
    }

    #[cfg(test)]
    pub(crate) fn set_application_commit_causality_limits(
        &self,
        batch_capacity: usize,
        byte_capacity: u64,
    ) {
        self.live_delivery.set_limits(batch_capacity, byte_capacity);
    }

    #[cfg(test)]
    pub(crate) fn set_application_commit_causality_reservation_hook(
        &self,
        hook: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    ) {
        self.live_delivery.set_after_reservation_hook(hook);
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_application_commit_causality(
        &self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
        retained_payload_bytes: u64,
    ) -> Result<
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
        &'static str,
    >{
        self.live_delivery.reserve(product, retained_payload_bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn publish_application_commit_causality(
        &self,
        reservation: crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        emissions: WorthQueryAdmittedApplicationEmissionBatch,
    ) -> usize {
        self.live_delivery
            .publish(reservation, publication, emissions)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn committed_application_emissions(
        &self,
        publication: &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission>
    {
        self.live_delivery.emissions(publication)
    }

    #[cfg(test)]
    pub(crate) fn published_application_commit_count(&self) -> usize {
        self.live_delivery.published_commit_count()
    }

    #[cfg(test)]
    pub(crate) fn retained_application_emission_bytes(&self) -> u64 {
        self.live_delivery.retained_payload_bytes()
    }

    #[cfg(test)]
    pub(crate) fn active_application_commit_causality_partitions(&self) -> usize {
        self.live_delivery.active_partition_count()
    }
}
