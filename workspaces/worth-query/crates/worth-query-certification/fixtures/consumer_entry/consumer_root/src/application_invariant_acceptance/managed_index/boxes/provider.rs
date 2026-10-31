//! The same controlled provider contract delegates bounds to the Query-free numerical owner.
use super::{coordinates::Coordinates, query};
use crate::application_invariant_acceptance::managed_index::schema::{Catalog, Item};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{build_box_tree, Box2, BoxIndexDenial, BoxNode};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

mod storage;
pub struct BoxIndex;
// compiled-example: begin
impl ApplicationManagedIndexBinding<Catalog> for BoxIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = super::coordinates::Binding;
    type Entity = Item;
    const NAME: &'static str = "catalog_box_index";
    const PROVIDER: &'static str = "managed-catalog.closed-box2.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}
// compiled-example: end
pub type Node = BoxNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<BoxIndexDenial>;
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub corrupt_summary: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub cold_lookup: std::sync::atomic::AtomicBool,
    capacity: AtomicUsize,
    builds: AtomicUsize,
    probe_validations: AtomicUsize,
    pub matched_sources: AtomicUsize,
    pub summary_comparisons: AtomicUsize,
    // Consumer observation after a genuine BVH node enters framework storage.
    pub after_first_node: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub coordinates: Mutex<[(u32, u64, u32); 16]>,
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
impl WorthQueryManagedIndexProvider<Catalog, BoxIndex> for Provider {
    fn decompose(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        control.checkpoint(1)?;
        Ok(match node.links {
            worth_query_consumer_values::BoxNodeLinks::Leaf { source } => {
                WorthQueryManagedIndexNodeRegion::new(Some(source), [None, None])
            }
            worth_query_consumer_values::BoxNodeLinks::Branch { left, right } => {
                WorthQueryManagedIndexNodeRegion::new(None, [Some(left), Some(right)])
            }
        })
    }

    type Node = Node;
    type Summary = Box2;
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = BoxIndex::PROVIDER;
    const VERSION: u32 = BoxIndex::PROVIDER_VERSION;
    fn source_summary(
        &self,
        _slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, BoxIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        let raw = row
            .field(query::bounds())
            .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
        Box2::new(raw.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))
    }
    fn merge_summaries(
        &self,
        parts: [Option<Self::Summary>; 3],
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, BoxIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(3)?;
        parts
            .into_iter()
            .flatten()
            .reduce(Box2::union)
            .ok_or(Stop::Domain(BoxIndexDenial::InvalidTopology))
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
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, BoxIndex>,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.summary_comparisons.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(4)?;
        let probe =
            Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(bounds.overlaps(probe))
    }

    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        self.0.capacity.fetch_add(1, Ordering::SeqCst);
        if source_count == 0 {
            return Ok(0);
        }
        source_count
            .checked_mul(2)
            .and_then(|count| count.checked_sub(1))
            .ok_or(WorthQueryManagedIndexProviderStop::Controlled(
                WorthQueryManagedIndexStop::CapacityOverflow,
            ))
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::SeqCst);
        assert!(
            sources.len() <= 16,
            "the fixture's actual coordinate observation is finite"
        );
        #[cfg(test)]
        if self.0.cold_lookup.load(Ordering::SeqCst) {
            nodes.before_source_node(sources.at(0)?.0)?;
        }
        let mut previous = None;
        for ordinal in 0..sources.len() {
            let (_, row) = sources.at(ordinal)?;
            let actual = row.entity_id();
            assert!(previous.is_none_or(|previous| previous < actual));
            previous = Some(actual);
            self.0.coordinates.lock().unwrap()[ordinal] = (
                actual.partition_value(),
                actual.local_slot_value(),
                actual.generation_value(),
            );
        }
        let root = build_box_tree(
            sources.len(),
            |ordinal| {
                let (slot, row) = sources.at(ordinal)?;
                let raw = row
                    .field(query::bounds())
                    .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
                let bounds = Box2::new(raw.raw())
                    .map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
                Ok((slot, bounds))
            },
            &mut storage::Storage {
                nodes,
                control,
                calls: &self.0,
            },
        )?;
        #[cfg(test)]
        if self.0.corrupt_summary.load(Ordering::SeqCst) {
            storage::corrupt_leaf(nodes, root)?;
        }
        Ok(root)
    }

    fn validate_probe(
        &self,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.probe_validations.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1)?;
        Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(())
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, BoxIndex>,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.matched_sources.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1)?;
        let raw = row
            .field(query::bounds())
            .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
        let bounds =
            Box2::new(raw.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        let probe =
            Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(bounds.overlaps(probe))
    }
}

/// A genuine incompatible executable version for the installation boundary twin.
pub struct IncompatibleVersion(pub Provider);
impl WorthQueryManagedIndexProvider<Catalog, BoxIndex> for IncompatibleVersion {
    type Node = Node;
    type Summary = Box2;
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = BoxIndex::PROVIDER;
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
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.build(sources, nodes, control)
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &Coordinates,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.query(sources, nodes, probe, candidates, control)
    }
}
