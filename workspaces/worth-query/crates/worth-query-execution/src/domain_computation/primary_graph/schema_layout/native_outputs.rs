//! Native output inventory lookup and its installed-tree cost.
use super::{AspectKey, WorthQueryPrimaryGraphLayout};
impl WorthQueryPrimaryGraphLayout {
    /// The installed native contract inventory is the complete output-aspect
    /// vocabulary for this application entity, including fields that were not
    /// written by the performed operation.
    pub(in crate::domain_computation::primary_graph) fn native_output_aspects<'a>(
        &'a self,
        entity: &'a str,
    ) -> impl Iterator<Item = &'a AspectKey> + Clone {
        self.aspect_contracts
            .get(entity)
            .into_iter()
            .flat_map(|contracts| contracts.keys())
    }

    /// Both the native kind and its aspect inventory use entity-name B-trees.
    /// The kind table contains every entity in the aspect table, so its bounded
    /// descent also covers the aspect table without reading unrelated aspects.
    pub(in crate::domain_computation::primary_graph) fn native_output_lookup_work(
        &self,
        entity: &str,
    ) -> Option<u64> {
        self.entity_kind_lookup_work(entity)?.checked_mul(2)
    }
}
