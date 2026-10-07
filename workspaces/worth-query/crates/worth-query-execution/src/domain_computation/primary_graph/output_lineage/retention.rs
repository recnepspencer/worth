use super::{
    prepared_slot::CancelledLineageSlot, ProductCoordinate, WorthQueryApplicationOutputLineage,
};

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.live_occurrences.remove(&occurrence);
        let mut retained = self.live_occurrences.clone();
        let mut frontier = retained.iter().copied().collect::<Vec<_>>();
        while let Some(child) = frontier.pop() {
            if let Some(parent) = self
                .origins
                .get(&child)
                .map(|coordinate| coordinate.occurrence)
            {
                if retained.insert(parent) {
                    frontier.push(parent);
                }
            }
        }
        let cancelled = &self.cancelled_slots;
        self.by_source.retain(|source, versions| {
            versions.retain(|indexed, history| {
                if retained.contains(indexed) {
                    return true;
                }
                history.retain(|generation, _| {
                    CancelledLineageSlot::contains_generation(
                        cancelled,
                        source,
                        ProductCoordinate {
                            occurrence: *indexed,
                            generation: *generation,
                        },
                    )
                });
                !history.is_empty()
            });
            !versions.is_empty()
        });
        self.partition_index
            .retain_occurrences(&retained, cancelled);
        self.origins.retain(|child, _| retained.contains(child));
    }
}

#[cfg(feature = "test-query-execution-observer")]
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Run a certification capacity event using real ledger custody. The
    /// callback holds no Query lock, and its temporary reservation refunds
    /// on return or unwind. This does not change installed resource limits.
    #[doc(hidden)]
    pub fn with_available_lineage_bytes_for_test<R>(
        &self,
        remaining: u64,
        run: impl FnOnce() -> R,
    ) -> Result<R, super::super::WorthQueryOutputDemandDenial> {
        let held = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retention
            .reserve_except_for_test(remaining)?;
        let result = run();
        drop(held);
        Ok(result)
    }

    /// The bytes this runtime's output lineage retains now: recorded outputs
    /// and the generation history that locates them.
    #[doc(hidden)]
    pub fn output_lineage_retained_bytes_for_test(&self) -> u64 {
        self.primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retention
            .retained_bytes()
    }
}
