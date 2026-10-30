//! Keyed box mechanics borrow Native-issued edge occurrences and endpoint regions.
use super::super::{bounds::Coordinates, schema::Catalog};
use super::{query, schema::Edge};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    insert_box_source, query_box_tree, remove_box_source, replace_box_source, Box2, BoxIndexDenial,
    BoxNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

mod storage;
pub struct BoxIndex;
impl ApplicationManagedIndexBinding<Catalog> for BoxIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = super::super::bounds::CoordinatesBinding;
    type Entity = Edge;
    const NAME: &'static str = "maintained_geometry_boxes";
    const PROVIDER: &'static str = "maintained-geometry.closed-box2.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
}
pub type Node = BoxNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<BoxIndexDenial>;
#[derive(Default)]
pub struct Calls {
    pub builds: AtomicUsize,
    pub updates: AtomicUsize,
    pub queries: AtomicUsize,
    pub source_bounds: AtomicUsize,
    pub navigation: AtomicUsize,
    pub after_node_write: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub after_build_node_write: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub last_decision: Mutex<Option<Vec<query::Row>>>,
}
pub struct Provider(pub Arc<Calls>);

fn bounds(
    sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
    slot: WorthQueryManagedIndexSourceSlot,
    calls: &Calls,
) -> Result<Box2, Stop> {
    calls.source_bounds.fetch_add(1, Ordering::SeqCst);
    let row = sources.get(slot)?;
    let a = row
        .one(query::start())
        .and_then(|row| row.field(query::start_region()))
        .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
    let b = row
        .one(query::end())
        .and_then(|row| row.field(query::end_region()))
        .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
    let a = Box2::new(a.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
    let b = Box2::new(b.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
    Ok(a.union(b))
}

impl WorthQueryManagedIndexProvider<Catalog, BoxIndex> for Provider {
    fn decompose(
        &self,
        _sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
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
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = BoxIndex::PROVIDER;
    const VERSION: u32 = BoxIndex::PROVIDER_VERSION;
    fn node_capacity(
        &self,
        count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        if count == 0 {
            return Ok(0);
        }
        count
            .checked_mul(2)
            .and_then(|count| count.checked_sub(1))
            .ok_or_else(|| WorthQueryManagedIndexStop::CapacityOverflow.into())
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::SeqCst);
        let mut root = None;
        for source in sources.slots() {
            let source = source?;
            root = Some(insert_box_source(
                root,
                source,
                bounds(sources, source, &self.0)?,
                &mut storage::Edit {
                    nodes,
                    control,
                    calls: &self.0,
                    observer: &self.0.after_build_node_write,
                },
            )?);
        }
        Ok(root)
    }
    fn update(
        &self,
        _before: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        for change in changes {
            let mut storage = storage::Edit {
                nodes,
                control,
                calls: &self.0,
                observer: &self.0.after_node_write,
            };
            match (change.before(), change.after()) {
                (Some(before), Some(after_slot)) if before == after_slot => {
                    replace_box_source(
                        root.ok_or_else(storage::invalid)?,
                        after_slot,
                        bounds(after, after_slot, &self.0)?,
                        &mut storage,
                    )?;
                }
                (before, after_slot) => {
                    if let Some(before) = before {
                        root = remove_box_source(
                            root.ok_or_else(storage::invalid)?,
                            before,
                            &mut storage,
                        )?;
                    }
                    if let Some(after_slot) = after_slot {
                        root = Some(insert_box_source(
                            root,
                            after_slot,
                            bounds(after, after_slot, &self.0)?,
                            &mut storage,
                        )?);
                    }
                }
            }
        }
        Ok(root)
    }
    fn query(
        &self,
        _sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, BoxIndex>,
        nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &Coordinates,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.queries.fetch_add(1, Ordering::SeqCst);
        let probe =
            Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        query_box_tree(
            nodes.root(),
            nodes.len(),
            probe,
            |id| {
                self.0.navigation.fetch_add(1, Ordering::SeqCst);
                nodes.get(id, control).copied().map_err(Into::into)
            },
            |source| candidates.emit(source).map_err(Into::into),
            |work| control.checkpoint(work).map_err(Into::into),
            storage::invalid,
        )
    }
}
