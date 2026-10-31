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
}
