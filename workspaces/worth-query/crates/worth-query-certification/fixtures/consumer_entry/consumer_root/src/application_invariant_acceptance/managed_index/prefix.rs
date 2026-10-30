//! Entry adapter: pure prefix mechanics borrow actual query rows and finite storage.
use super::{query, schema::Catalog};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    insert_prefix_source, query_prefix_tree, remove_prefix_source, PrefixIndexDenial, PrefixNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

#[path = "prefix/storage.rs"]
mod storage;
#[path = "prefix/structure.rs"]
pub(super) mod structure;
pub struct PrefixIndex;
// compiled-example: begin
impl ApplicationManagedIndexBinding<Catalog> for PrefixIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = super::schema::Item;
    const NAME: &'static str = "catalog_prefix_index";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
}
// compiled-example: end
pub type Node = PrefixNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<PrefixIndexDenial>;
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub(super) structure_fault: Mutex<structure::Fault>,
    pub capacity: AtomicUsize,
    pub builds: AtomicUsize,
    pub queries: AtomicUsize,
    pub updates: AtomicUsize,
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
    fn decompose(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        structure::describe(&self.0, sources, node, control)
    }
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
        for (ordinal, slot) in sources.slots().enumerate() {
            let row = sources.get(slot?)?;
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
        #[cfg(test)]
        if *self.0.structure_fault.lock().unwrap() == structure::Fault::DisconnectedCycle {
            return structure::cyclic_sources(sources, nodes);
        }
        let mut storage = storage::Build {
            sources,
            nodes,
            control,
            after_first_node: &self.0.after_first_node,
        };
        let mut root = None;
        for slot in sources.slots() {
            root = Some(insert_prefix_source(root, slot?, &mut storage)?);
        }
        Ok(root)
    }
    fn update(
        &self,
        before: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        // All removals see the committed value order; only then may proposed
        // values participate. The pages belong to this isolated staged edit.
        for change in changes {
            if let Some(slot) = change.before() {
                root = remove_prefix_source(
                    root,
                    slot,
                    &mut storage::Build {
                        sources: before,
                        nodes,
                        control,
                        after_first_node: &self.0.after_first_node,
                    },
                )?;
            }
        }
        for change in changes {
            if let Some(slot) = change.after() {
                root = Some(insert_prefix_source(
                    root,
                    slot,
                    &mut storage::Build {
                        sources: after,
                        nodes,
                        control,
                        after_first_node: &self.0.after_first_node,
                    },
                )?);
            }
        }
        Ok(root)
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
