//! Scene-specific projection adapter; Native owns Source lifetime and publication.
use super::box_query::{SceneBoxProbe, SceneBoxProbeBinding};
use super::*;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_query_consumer_values::{
    Box2Coordinates, BoxIndexDenial, BoxNode, BoxNodeLinks, IntegerBox2, ModelLocalMetre,
};
use worth_query_decl::facade::application_query::*;
use worth_query_host::facade::primary_graph::*;
mod edits;
mod predicate;
mod storage;

const PROVIDER: &str = "worth.query.scene-face.closed-aabb-model-local.v1";
const VERSION: u32 = 1;
pub struct SceneFaceBoxIndex<S>(PhantomData<fn() -> S>);
impl<S: SceneFaceSchemaBinding> ApplicationManagedIndexBinding<S> for SceneFaceBoxIndex<S> {
    type SourceBinding = SceneFacesReadBinding<S>;
    type ProbeBinding = SceneBoxProbeBinding;
    type Entity = Body;
    const NAME: &'static str = "scene_face_box_candidates";
    const PROVIDER: &'static str = self::PROVIDER;
    const PROVIDER_VERSION: u32 = VERSION;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}
/// Passive observations of the actual installed adapter; no control or authority.
#[derive(Default)]
pub struct SceneBoxObservations {
    pub builds: AtomicUsize,
    pub updates: AtomicUsize,
    pub probes: AtomicUsize,
}
#[derive(Default)]
pub struct SceneFaceConfiguration {
    pub observations: Arc<SceneBoxObservations>,
}
pub(super) struct Provider(pub Arc<SceneBoxObservations>);
type Bounds = IntegerBox2<ModelLocalMetre>;
type Node =
    BoxNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot, ModelLocalMetre>;
type Stop = WorthQueryManagedIndexProviderStop<BoxIndexDenial>;

fn probe_bounds(probe: &SceneBoxProbe) -> Result<Bounds, Stop> {
    Bounds::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))
}
fn row_bounds<S: SceneFaceSchemaBinding>(
    row: WorthQueryManagedIndexSourceRow<'_, S, SceneFaceBoxIndex<S>>,
) -> Result<Bounds, Stop> {
    let points = query::points(&row).map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
    let boxes = points.map(|point| {
        Bounds::new(Box2Coordinates {
            lower: point,
            upper: point,
        })
        .expect("a point has valid closed bounds")
    });
    Ok(boxes[0].union(boxes[1]).union(boxes[2]))
}
impl<S: SceneFaceSchemaBinding> WorthQueryManagedIndexProvider<S, SceneFaceBoxIndex<S>>
    for Provider
{
    type Node = Node;
    type Summary = Bounds;
    type Denial = BoxIndexDenial;
    const IDENTITY: &'static str = PROVIDER;
    const VERSION: u32 = self::VERSION;
    fn decompose(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        control.checkpoint(1)?;
        Ok(match node.links {
            BoxNodeLinks::Leaf { source } => {
                WorthQueryManagedIndexNodeRegion::new(Some(source), [None, None])
            }
            BoxNodeLinks::Branch { left, right } => {
                WorthQueryManagedIndexNodeRegion::new(None, [Some(left), Some(right)])
            }
        })
    }
    fn source_summary(
        &self,
        _slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, SceneFaceBoxIndex<S>>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Bounds, Stop> {
        control.checkpoint(1)?;
        row_bounds(row)
    }
    fn merge_summaries(
        &self,
        parts: [Option<Bounds>; 3],
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, SceneFaceBoxIndex<S>>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Bounds, Stop> {
        control.checkpoint(3)?;
        parts
            .into_iter()
            .flatten()
            .reduce(Bounds::union)
            .ok_or(Stop::Domain(BoxIndexDenial::InvalidTopology))
    }
    fn node_summary(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Bounds, Stop> {
        control.checkpoint(1)?;
        Ok(node.bounds)
    }
    fn summary_may_match(
        &self,
        bounds: Bounds,
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, SceneFaceBoxIndex<S>>,
        probe: &SceneBoxProbe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(4)?;
        Ok(bounds.overlaps(probe_bounds(probe)?))
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
            .and_then(|n| n.checked_sub(1))
            .ok_or_else(|| WorthQueryManagedIndexStop::CapacityOverflow.into())
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, SceneFaceBoxIndex<S>>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::SeqCst);
        edits::build(sources, nodes, control)
    }
    fn update(
        &self,
        _before: &WorthQueryManagedIndexSourceRows<'_, S, SceneFaceBoxIndex<S>>,
        after: &WorthQueryManagedIndexSourceRows<'_, S, SceneFaceBoxIndex<S>>,
        changes: &[WorthQueryManagedIndexSourceChange],
        root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        edits::update(after, changes, root, nodes, control)
    }
    fn validate_probe(
        &self,
        probe: &SceneBoxProbe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.probes.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1)?;
        probe_bounds(probe).map(drop)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, SceneFaceBoxIndex<S>>,
        probe: &SceneBoxProbe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(4)?;
        Ok(row_bounds(row)?.overlaps(probe_bounds(probe)?))
    }
}
