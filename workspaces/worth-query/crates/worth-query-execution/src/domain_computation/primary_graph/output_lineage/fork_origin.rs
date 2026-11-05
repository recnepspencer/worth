//! Captured fork origins; registration copies no computation state.
use {
    super::WorthQueryApplicationOutputLineage,
    crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate,
};
impl WorthQueryApplicationOutputLineage {
    pub(crate) fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        let source = ProductCoordinate {
            occurrence: source.lifecycle_incarnation(),
            generation: source.reference_generation().get(),
        };
        let destination = destination.lifecycle_incarnation();
        self.live_occurrences.insert(source.occurrence);
        self.live_occurrences.insert(destination);
        assert!(
            self.origins.insert(destination, source).is_none(),
            "one product occurrence may be registered as a fork once"
        );
    }
}

mod court_doors_for_test;
