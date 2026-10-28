//! The same controlled provider contract delegates bounds to the Query-free numerical owner.
use super::{coordinates::Coordinates, query};
use crate::application_invariant_acceptance::managed_index::schema::{Catalog, Item};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{build_box_tree, query_box_tree, Box2, BoxIndexDenial, BoxNode};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

pub struct BoxIndex;
// compiled-example: begin
impl ApplicationManagedIndexBinding<Catalog> for BoxIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = super::coordinates::Binding;
    type Entity = Item;
    const NAME: &'static str = "catalog_box_index";
    const PROVIDER: &'static str = "managed-catalog.closed-box2.v1";
    const PROVIDER_VERSION: u32 = 1;
}
// compiled-example: end
pub type Node = BoxNode<WorthQueryManagedIndexNodeId>;
pub type Stop = WorthQueryManagedIndexProviderStop<BoxIndexDenial>;
#[derive(Default)]
pub struct Calls {
    capacity: AtomicUsize,
    builds: AtomicUsize,
    queries: AtomicUsize,
    pub navigation: AtomicUsize,
    pub coordinates: Mutex<[(u32, u64, u32); 16]>,
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
impl WorthQueryManagedIndexProvider<Catalog, BoxIndex> for Provider {
    type Node = Node;
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = BoxIndex::PROVIDER;
    const VERSION: u32 = BoxIndex::PROVIDER_VERSION;
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
        build_box_tree(
            sources.len(),
            |ordinal| {
                let (_, row) = sources.at(ordinal)?;
                let raw = row
                    .field(query::bounds())
                    .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
                Box2::new(raw.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))
            },
            |node| nodes.push(node).map_err(Stop::Controlled),
            |work| control.checkpoint(work).map_err(Stop::Controlled),
        )
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &Coordinates,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.queries.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1)?;
        let probe =
            Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        query_box_tree(
            nodes.root(),
            nodes.len(),
            probe,
            |id| {
                self.0.navigation.fetch_add(1, Ordering::SeqCst);
                nodes.get(id, control).copied().map_err(Stop::Controlled)
            },
            |ordinal| {
                let (slot, _) = sources.at(ordinal)?;
                candidates.emit(slot).map_err(Stop::Controlled)
            },
            |work| control.checkpoint(work).map_err(Stop::Controlled),
            || Stop::Domain(BoxIndexDenial::InvalidTopology),
        )
    }
}

/// A genuine incompatible executable version for the installation boundary twin.
pub struct IncompatibleVersion(pub Provider);
impl WorthQueryManagedIndexProvider<Catalog, BoxIndex> for IncompatibleVersion {
    type Node = Node;
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
