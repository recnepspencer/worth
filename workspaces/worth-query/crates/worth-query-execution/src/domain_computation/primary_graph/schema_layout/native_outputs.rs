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
}
