//! Entry adapter: pure prefix mechanics borrow actual query rows and finite storage.
use super::{query, schema::Catalog};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    build_prefix_tree, query_prefix_tree, PrefixIndexDenial, PrefixNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

mod storage;
pub struct PrefixIndex;
// compiled-example: begin
impl ApplicationManagedIndexBinding<Catalog> for PrefixIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = super::schema::Item;
    const NAME: &'static str = "catalog_prefix_index";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
}
// compiled-example: end
pub type Node = PrefixNode<WorthQueryManagedIndexNodeId>;
pub type Stop = WorthQueryManagedIndexProviderStop<PrefixIndexDenial>;
#[derive(Default)]
pub struct Calls {
    pub capacity: AtomicUsize,
    pub builds: AtomicUsize,
    pub queries: AtomicUsize,
    pub canonical_rows: AtomicUsize,
    // Consumer fixture observation, invoked only after a genuine first node write.
    pub after_first_node: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub canonical_coordinates: Mutex<[(u32, u64, u32); 8]>,
}
impl Calls {
    pub fn totals(&self) -> (usize, usize, usize) {
        (
            self.capacity.load(Ordering::SeqCst),
            self.builds.load(Ordering::SeqCst),
            self.queries.load(Ordering::SeqCst),
        )
    }
}
pub struct Provider(pub Arc<Calls>);
impl WorthQueryManagedIndexProvider<Catalog, PrefixIndex> for Provider {
    type Node = Node;
    type Denial = PrefixIndexDenial;
    const IDENTITY: &'static str = PrefixIndex::PROVIDER;
    const VERSION: u32 = PrefixIndex::PROVIDER_VERSION;
    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        self.0.capacity.fetch_add(1, Ordering::SeqCst);
        Ok(source_count)
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::SeqCst);
        // This assertion uses genuine full-GEN row identities, never their labels.
        // The independent ordinary Source query deliberately has another order.
        let mut previous = None;
        assert!(
            sources.len() <= 8,
            "the literal fixture observation is finite"
        );
        for ordinal in 0..sources.len() {
            let (_, row) = sources.at(ordinal)?;
            let actual = row.entity_id();
            assert!(previous.is_none_or(|previous| previous < actual));
            previous = Some(actual);
            self.0.canonical_coordinates.lock().unwrap()[ordinal] = (
                actual.partition_value(),
                actual.local_slot_value(),
                actual.generation_value(),
            );
        }
        self.0.canonical_rows.store(sources.len(), Ordering::SeqCst);
        build_prefix_tree(
            0..sources.len(),
            &mut storage::Build {
                sources,
                nodes,
                control,
                after_first_node: &self.0.after_first_node,
            },
        )
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &query::Probe,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.queries.fetch_add(1, Ordering::SeqCst);
        query_prefix_tree(
            nodes.root(),
            nodes.len(),
            &mut storage::Query {
                sources,
                nodes,
                probe,
                candidates,
                control,
            },
        )
    }
}

/// A genuine incompatible executable version for the installation boundary twin.
pub struct IncompatibleVersion(pub Provider);
impl WorthQueryManagedIndexProvider<Catalog, PrefixIndex> for IncompatibleVersion {
    type Node = Node;
    type Denial = PrefixIndexDenial;
    const IDENTITY: &'static str = PrefixIndex::PROVIDER;
    const VERSION: u32 = 2;
    fn node_capacity(
        &self,
        count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        self.0.node_capacity(count, control)
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
