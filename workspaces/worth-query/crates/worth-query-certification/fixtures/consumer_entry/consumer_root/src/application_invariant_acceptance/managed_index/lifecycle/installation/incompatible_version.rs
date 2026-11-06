use super::super::super::{
    prefix::{Node, PrefixIndex, Provider, Stop},
    query,
    schema::Catalog,
};
use worth_query_consumer_values::PrefixIndexDenial;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

/// A genuine incompatible executable version for the installation boundary twin.
pub struct IncompatibleVersion(pub Provider);
impl WorthQueryManagedIndexProvider<Catalog, PrefixIndex> for IncompatibleVersion {
    type Node = Node;
    type Summary =
        worth_query_consumer_values::PrefixSourceBounds<WorthQueryManagedIndexSourceSlot>;
    type Denial = PrefixIndexDenial;
    const IDENTITY: &'static str = PrefixIndex::PROVIDER;
    const VERSION: u32 = 2;
    fn node_capacity(
        &self,
        count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        <Provider as WorthQueryManagedIndexProvider<Catalog, PrefixIndex>>::node_capacity(
            &self.0, count, control,
        )
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.build(sources, nodes, control)
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &query::Probe,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.query(sources, nodes, probe, candidates, control)
    }
}
