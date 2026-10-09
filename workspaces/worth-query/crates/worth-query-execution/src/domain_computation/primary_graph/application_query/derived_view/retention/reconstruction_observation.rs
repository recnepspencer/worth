//! Read-only observation of actual retained membership and entry dependencies.
use super::WorthQueryManagedDerivedView;
pub(in crate::domain_computation::primary_graph) type RetentionObservation =
    (bool, usize, Vec<String>, Vec<String>);
impl<Query, Key, Value> WorthQueryManagedDerivedView<Query, Key, Value> {
    pub(in crate::domain_computation::primary_graph) fn reconstruction_retention_for_test(
        &self,
    ) -> RetentionObservation {
        let retained = self
            .state
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let membership = retained
            .membership
            .iter()
            .map(|fact| format!("{fact:?}"))
            .collect();
        let mut entries = retained
            .entries
            .values()
            .flat_map(|entry| entry.dependencies.iter().map(|fact| format!("{fact:?}")))
            .collect::<Vec<_>>();
        entries.sort();
        (retained.cold, retained.entries.len(), membership, entries)
    }
}
