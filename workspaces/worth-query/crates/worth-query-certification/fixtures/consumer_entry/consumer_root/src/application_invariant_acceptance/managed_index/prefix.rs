//! Entry adapter: pure prefix mechanics borrow actual query rows and finite storage.
use super::{query, schema::Catalog};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    insert_prefix_source, remove_prefix_source, PrefixIndexDenial, PrefixNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

#[path = "prefix/storage.rs"]
mod storage;
#[path = "prefix/structure.rs"]
pub(super) mod structure;
#[path = "prefix/summary.rs"]
mod summary;
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
    const MEMBERSHIP:
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        super::PREFIX_MEMBERSHIP;
}
// compiled-example: end
pub type Node = PrefixNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<PrefixIndexDenial>;
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub summary_read_first: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(super) first_source: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    #[cfg(test)]
    pub(super) after_second_match: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub(super) repeat_candidates: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(super) foreign_candidate: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    pub matched_sources: AtomicUsize,
    pub summary_comparisons: AtomicUsize,
    #[cfg(test)]
    pub(super) structure_fault: Mutex<structure::Fault>,
    pub capacity: AtomicUsize,
    pub builds: AtomicUsize,
    pub probe_validations: AtomicUsize,
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
            self.probe_validations.load(Ordering::SeqCst),
        )
    }
}
pub struct Provider(pub Arc<Calls>);
impl WorthQueryManagedIndexProvider<Catalog, PrefixIndex> for Provider {
    fn decompose(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        structure::describe(&self.0, node, control)
    }
    type Node = Node;
    type Summary =
        worth_query_consumer_values::PrefixSourceBounds<WorthQueryManagedIndexSourceSlot>;
    type Denial = PrefixIndexDenial;
    const IDENTITY: &'static str = PrefixIndex::PROVIDER;
    const VERSION: u32 = PrefixIndex::PROVIDER_VERSION;
    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        _row: WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        Ok(worth_query_consumer_values::PrefixSourceBounds::point(slot))
    }
    fn merge_summaries(
        &self,
        parts: [Option<Self::Summary>; 3],
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, PrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        #[cfg(test)]
        if self.0.summary_read_first.load(Ordering::SeqCst) {
            sources.get(
                self.0
                    .first_source
                    .lock()
                    .unwrap()
                    .expect("the build observed a genuine slot"),
            )?;
        }
        summary::merge(parts, sources, control)
    }
    fn node_summary(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        Ok(node.bounds)
    }
    fn summary_may_match(
        &self,
        bounds: Self::Summary,
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, PrefixIndex>,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.summary_comparisons.fetch_add(1, Ordering::SeqCst);
        summary::may_match(bounds, sources, probe, control)
    }
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
            let slot = slot?;
            #[cfg(test)]
            if ordinal == 0 {
                *self.0.first_source.lock().unwrap() = Some(slot);
            }
            let row = sources.get(slot)?;
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
        #[cfg(test)]
        if *self.0.structure_fault.lock().unwrap() == structure::Fault::DuplicateChild {
            // A genuine value-only write of the exact predecessor node. Its
            // first child is therefore a retained edge when describe duplicates
            // it. No pure edit can replace that child before the hostile check.
            if let Some(root) = root {
                let node = nodes.get(root)?;
                nodes.replace(root, node)?;
            }
            return Ok(root);
        }
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
        _nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &query::Probe,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.validate_probe(probe, control)?;
        assert!(candidates.is_empty());
        #[cfg(test)]
        if let Some(slot) = *self.0.foreign_candidate.lock().unwrap() {
            candidates.emit(slot)?;
        }
        let mut count = 0;
        for ordinal in (0..sources.len()).rev() {
            let (slot, row) = sources.at(ordinal)?;
            if self.matches_source(row, probe, control)? {
                candidates.emit(slot)?;
                #[cfg(test)]
                if self.0.repeat_candidates.load(Ordering::SeqCst) {
                    candidates.emit(slot)?;
                    candidates.emit(slot)?;
                }
                count += 1;
                assert_eq!(candidates.len(), count);
            }
        }
        assert_eq!(candidates.is_empty(), count == 0);
        Ok(())
    }
    fn validate_probe(
        &self,
        _probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.probe_validations.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1).map_err(Into::into)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let name = row
            .field(query::name())
            .map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))?;
        let order = worth_query_consumer_values::prefix_order(&name, &probe.prefix, |units| {
            control.checkpoint(units).map_err(Stop::Controlled)
        })?;
        let previous = self.0.matched_sources.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        if previous == 1 {
            if let Some(observe) = self.0.after_second_match.lock().unwrap().take() {
                observe();
            }
        }
        #[cfg(not(test))]
        let _ = previous;
        Ok(order == worth_query_consumer_values::PrefixOrder::Matches)
    }
}

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
