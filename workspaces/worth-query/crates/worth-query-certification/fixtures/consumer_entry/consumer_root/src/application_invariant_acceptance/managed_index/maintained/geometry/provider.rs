//! Keyed box mechanics borrow Native-issued edge occurrences and endpoint regions.
use super::super::{
    bounds::{Coordinates, CoordinatesBinding},
    schema::Catalog,
};
use super::{query, schema::Edge};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    insert_box_source, remove_box_source, replace_box_source, Box2, BoxIndexDenial, BoxNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

#[cfg(test)]
pub(super) mod cost_stages;
mod predicate;
mod refinement;
mod storage;
#[cfg(test)]
mod summary_fault;
pub(super) const PROVIDER_IDENTITY: &str = "maintained-geometry.closed-box2.v1";
pub(super) const PROVIDER_VERSION: u32 = 1;
pub struct BoxIndex;
impl ApplicationManagedIndexBinding<Catalog> for BoxIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = super::super::bounds::CoordinatesBinding;
    type Entity = Edge;
    const NAME: &'static str = "maintained_geometry_boxes";
    const PROVIDER: &'static str = PROVIDER_IDENTITY;
    const PROVIDER_VERSION: u32 = self::PROVIDER_VERSION;
    const REFINEMENT: Option<
        worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
    > = Some(
        worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement::row_local(
            "maintained-geometry.closed-segment-box",
            1,
        ),
    );
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}
pub type Material = BoxNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
#[derive(Clone)]
pub struct Node {
    material: Material,
    summary: Box<Box2>,
}
impl From<Material> for Node {
    fn from(material: Material) -> Self {
        Self {
            summary: Box::new(material.bounds),
            material,
        }
    }
}
pub type Stop = WorthQueryManagedIndexProviderStop<BoxIndexDenial>;
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub(super) cost_stages: cost_stages::CostStages,
    #[cfg(test)]
    pub preserve_root: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub write_leaves_only: std::sync::atomic::AtomicBool,
    pub builds: AtomicUsize,
    pub updates: AtomicUsize,
    pub probe_validations: AtomicUsize,
    pub matched_sources: AtomicUsize,
    pub summary_comparisons: AtomicUsize,
    /// Actual bounds acquisitions by pure tree construction/maintenance.
    pub source_bounds: AtomicUsize,
    pub summary_sources: AtomicUsize,
    pub navigation: AtomicUsize,
    pub after_node_write: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub after_build_node_write: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub last_decision: Mutex<Option<Vec<query::Row>>>,
}
pub struct Provider(pub Arc<Calls>);

fn bounds<Index>(
    sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
    slot: WorthQueryManagedIndexSourceSlot,
    calls: &Calls,
) -> Result<Box2, Stop>
where
    Index: ApplicationManagedIndexBinding<
        Catalog,
        SourceBinding = query::SourceBinding,
        ProbeBinding = CoordinatesBinding,
        Entity = Edge,
    >,
{
    calls.source_bounds.fetch_add(1, Ordering::SeqCst);
    row_bounds(sources.get(slot)?)
}
fn row_bounds<Index>(row: WorthQueryManagedIndexSourceRow<'_, Catalog, Index>) -> Result<Box2, Stop>
where
    Index: ApplicationManagedIndexBinding<
        Catalog,
        SourceBinding = query::SourceBinding,
        ProbeBinding = CoordinatesBinding,
        Entity = Edge,
    >,
{
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

impl<Index> WorthQueryManagedIndexProvider<Catalog, Index> for Provider
where
    Index: ApplicationManagedIndexBinding<
        Catalog,
        SourceBinding = query::SourceBinding,
        ProbeBinding = CoordinatesBinding,
        Entity = Edge,
    >,
{
    fn decompose(
        &self,
        node: &Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::Decompose);
        control.checkpoint(1)?;
        Ok(match node.material.links {
            worth_query_consumer_values::BoxNodeLinks::Leaf { source } => {
                WorthQueryManagedIndexNodeRegion::new(Some(source), [None, None])
            }
            worth_query_consumer_values::BoxNodeLinks::Branch { left, right } => {
                WorthQueryManagedIndexNodeRegion::new(None, [Some(left), Some(right)])
            }
        })
    }

    type Node = Node;
    type Summary = Box<Box2>;
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = PROVIDER_IDENTITY;
    const VERSION: u32 = PROVIDER_VERSION;
    fn source_summary(
        &self,
        _slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, Index>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::Summary);
        control.checkpoint(1)?;
        self.0.summary_sources.fetch_add(1, Ordering::SeqCst);
        row_bounds(row).map(Box::new)
    }
    fn merge_summaries(
        &self,
        parts: [Option<&Self::Summary>; 3],
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, Index>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::Summary);
        control.checkpoint(3)?;
        parts
            .into_iter()
            .flatten()
            .map(|bounds| **bounds)
            .reduce(Box2::union)
            .map(Box::new)
            .ok_or(Stop::Domain(BoxIndexDenial::InvalidTopology))
    }
    fn node_summary<'node>(
        &self,
        node: &'node Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&'node Self::Summary, Stop> {
        control.checkpoint(1)?;
        Ok(&node.summary)
    }
    fn summary_may_match(
        &self,
        bounds: &Self::Summary,
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, Index>,
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
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
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
        _before: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
        after: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::UpdateBegin);
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        if self.0.preserve_root.load(Ordering::SeqCst) {
            return Ok(root);
        }
        #[cfg(test)]
        if self.0.write_leaves_only.load(Ordering::SeqCst) {
            summary_fault::replace_leaves(after, changes, nodes, &self.0)?;
            return Ok(root);
        }
        for change in changes {
            let mut storage = storage::Edit {
                nodes,
                control,
                calls: &self.0,
                observer: &self.0.after_node_write,
            };
            match (change.before(), change.after()) {
                (Some(before), Some(after_slot)) if before == after_slot => {
                    root = Some(replace_box_source(
                        root.ok_or_else(storage::invalid)?,
                        after_slot,
                        bounds(after, after_slot, &self.0)?,
                        &mut storage,
                    )?);
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
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::UpdateEnd);
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
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, Index>,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.matched_sources.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(4)?;
        let probe =
            Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(row_bounds(row)?.overlaps(probe))
    }
}
