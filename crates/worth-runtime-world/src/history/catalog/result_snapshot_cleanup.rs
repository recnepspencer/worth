use super::*;

impl CompositeHistoryCatalog {
    pub(crate) fn owned_relational_results(
        &self,
    ) -> Vec<Arc<worth_relational::facade::transactions::CommitResult>> {
        let state = lock_state(&self.state);
        state
            .entries
            .values()
            .filter_map(|slot| {
                slot.get()?
                    .publication
                    .as_ref()?
                    .facts()?
                    .component_results
                    .owned_relational_result()
            })
            .collect()
    }

    pub(crate) fn has_undelivered_publication(&self) -> bool {
        let state = lock_state(&self.state);
        state.entries.values().any(|slot| {
            slot.get()
                .and_then(|entry| entry.publication.as_ref())
                .is_some_and(|publication| publication.awaits_delivery())
        })
    }
}
