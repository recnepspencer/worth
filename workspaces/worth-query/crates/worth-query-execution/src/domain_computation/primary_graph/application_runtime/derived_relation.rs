//! Disposal and sealed lifecycle inspection of framework-owned recipe projections.
pub use super::super::application_query::derived_view::WorthQueryRuleReadableCollectionLifecycleForTest;
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Discards rebuildable rule-readable collection projections only. Future
    /// reads obtain their source afresh and may reconstruct the projection.
    /// This changes no canonical truth, rule proof or ordinary Query view.
    pub fn discard_rule_readable_collections(&self) {
        if let Some(graph) = self.runtime.primary_graph() {
            graph.managed_derived_views().discard_native_counts();
        }
    }

    /// Inspect actual registered lifecycle states without exposing source keys
    /// or values. This is a closed read-only certification door.
    #[doc(hidden)]
    pub fn rule_readable_collection_lifecycle_for_test(
        &self,
    ) -> Vec<WorthQueryRuleReadableCollectionLifecycleForTest> {
        self.runtime.primary_graph().map_or_else(Vec::new, |graph| {
            graph.managed_derived_views().native_count_lifecycle()
        })
    }
}
