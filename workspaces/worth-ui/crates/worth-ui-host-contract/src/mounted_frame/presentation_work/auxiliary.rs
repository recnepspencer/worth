use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{UiMountedPaintCommand, UiMountedPaintCommandIdentity};

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedPresentationAuxiliaryState {
    frame: crate::UiMountedFrameIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
    binding: crate::UiSurfaceBindingGeneration,
    content: crate::UiMountedContentGeneration,
    nodes: Arc<[crate::UiMountedNodeProjectionView]>,
    portal_overlays: crate::UiMountedPortalOverlayTable,
    semantic_text: crate::UiMountedSemanticTextTable,
    clips: crate::UiMountedClipTable,
    layers: crate::UiMountedLayerTable,
    hit_tests: crate::UiMountedHitTestTable,
    paint_batches: crate::UiMountedPaintBatchTable,
    spatial_batches: crate::UiMountedSpatialBatchTable,
    realtime_batches: crate::UiMountedRealtimeBatchTable,
    resources: crate::UiMountedResourceTable,
    authored_commands: Arc<[UiMountedPaintCommandIdentity]>,
    authored_order: Arc<[crate::UiMountedPaintOrderIdentity]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPresentationReconstructionDenial {
    DuplicateTableIndex,
    MissingTableIndex,
    CommandMismatch,
    CapacityExceeded,
}

impl UiMountedPresentationAuxiliaryState {
    #[doc(hidden)]
    pub fn from_runtime_mounting(projection: &crate::UiMountedProjectionView) -> Self {
        Self {
            frame: projection.frame(),
            surface: projection.surface(),
            binding: projection.binding(),
            content: projection.content_generation(),
            nodes: projection.retained_nodes(),
            portal_overlays: projection.portal_overlays().clone(),
            semantic_text: projection.semantic_text().clone(),
            clips: projection.clips().clone(),
            layers: projection.layers().clone(),
            hit_tests: projection.hit_tests().clone(),
            paint_batches: projection.paint_batches().clone(),
            spatial_batches: projection.spatial_batches().clone(),
            realtime_batches: projection.realtime_batches().clone(),
            resources: projection.resources().clone(),
            authored_commands: projection
                .retained_paint_commands()
                .iter()
                .map(UiMountedPaintCommand::identity)
                .collect(),
            authored_order: projection.retained_paint_order(),
        }
    }

    pub fn reconstruct(
        &self,
        commands: &HashMap<UiMountedPaintCommandIdentity, UiMountedPaintCommand>,
    ) -> Result<crate::UiMountedProjectionView, UiMountedPresentationReconstructionDenial> {
        validate_commands(commands, &self.portal_overlays, &self.semantic_text)?;
        Ok(crate::UiMountedProjectionView::new(
            crate::UiMountedProjectionViewInput {
                frame: self.frame,
                surface: self.surface,
                binding: self.binding,
                content_generation: self.content,
                nodes: self.nodes.to_vec(),
                clips: self.clips.clone(),
                layers: self.layers.clone(),
                portal_overlays: self.portal_overlays.clone(),
                semantic_text: self.semantic_text.clone(),
                hit_tests: self.hit_tests.clone(),
                paint_batches: self.paint_batches.clone(),
                spatial_batches: self.spatial_batches.clone(),
                realtime_batches: self.realtime_batches.clone(),
                resources: self.resources.clone(),
                authored_paint_commands: self
                    .authored_commands
                    .iter()
                    .map(|identity| {
                        commands
                            .get(identity)
                            .expect("validated reconstruction source names a command")
                            .clone()
                    })
                    .collect(),
                authored_paint_order: self.authored_order.to_vec(),
            },
        ))
    }

    #[doc(hidden)]
    pub fn reconstruct_authored(
        &self,
    ) -> Result<crate::UiMountedProjectionView, UiMountedPresentationReconstructionDenial> {
        let commands = self
            .portal_overlays
            .rows()
            .iter()
            .copied()
            .map(|mechanic| UiMountedPaintCommand::PortalOverlay {
                identity: UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
                mechanic,
            })
            .chain(self.semantic_text.rows().iter().cloned().map(|mechanic| {
                UiMountedPaintCommand::SemanticText {
                    identity: UiMountedPaintCommandIdentity::semantic_text(&mechanic),
                    mechanic,
                }
            }))
            .map(|command| (command.identity(), command))
            .collect::<HashMap<_, _>>();
        self.reconstruct(&commands)
    }

    #[doc(hidden)]
    pub fn same_presentation_meaning(&self, other: &Self) -> bool {
        self.surface == other.surface
            && self.binding == other.binding
            && same_nodes(&self.nodes, &other.nodes)
            && self.clips == other.clips
            && self.layers == other.layers
            && self.portal_overlays.schema() == other.portal_overlays.schema()
            && self.portal_overlays.rows().len() == other.portal_overlays.rows().len()
            && self
                .portal_overlays
                .rows()
                .iter()
                .zip(other.portal_overlays.rows())
                .all(|(previous, next)| previous.same_retained_paint_meaning(*next))
            && same_hit_tests(&self.hit_tests, &other.hit_tests)
            && self.paint_batches == other.paint_batches
            && self.spatial_batches == other.spatial_batches
            && self.realtime_batches == other.realtime_batches
            && self.resources == other.resources
    }

    #[doc(hidden)]
    pub fn same_lane_presentation_meaning(&self, other: &Self) -> bool {
        self.paint_batches == other.paint_batches
            && self.spatial_batches == other.spatial_batches
            && self.realtime_batches == other.realtime_batches
            && self.resources == other.resources
    }

    #[doc(hidden)]
    pub fn refresh_lane_presentation_from_runtime_mounting(
        &mut self,
        frame: crate::UiMountedFrameIdentity,
        surface: crate::UiSemanticSurfaceIdentity,
        binding: crate::UiSurfaceBindingGeneration,
        content: crate::UiMountedContentGeneration,
        paint_batches: crate::UiMountedPaintBatchTable,
        spatial_batches: crate::UiMountedSpatialBatchTable,
        realtime_batches: crate::UiMountedRealtimeBatchTable,
        resources: crate::UiMountedResourceTable,
    ) {
        self.frame = frame;
        self.surface = surface;
        self.binding = binding;
        self.content = content;
        self.paint_batches = paint_batches;
        self.spatial_batches = spatial_batches;
        self.realtime_batches = realtime_batches;
        self.resources = resources;
    }

    pub const fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.frame
    }

    pub const fn surface(&self) -> crate::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub const fn binding(&self) -> crate::UiSurfaceBindingGeneration {
        self.binding
    }

    pub const fn content(&self) -> crate::UiMountedContentGeneration {
        self.content
    }

    pub fn paint_batch_count(&self) -> usize {
        self.paint_batches.rows().len()
    }

    pub fn node_receipt_for(
        &self,
        mounted_instance: crate::UiMountedInstanceIdentity,
    ) -> Option<crate::UiMountedNodeReceiptIdentity> {
        self.nodes
            .iter()
            .find(|node| node.mounted_instance() == mounted_instance)
            .map(crate::UiMountedNodeProjectionView::node_receipt)
    }
}

fn validate_commands(
    commands: &HashMap<UiMountedPaintCommandIdentity, UiMountedPaintCommand>,
    portal_overlays: &crate::UiMountedPortalOverlayTable,
    semantic_text: &crate::UiMountedSemanticTextTable,
) -> Result<(), UiMountedPresentationReconstructionDenial> {
    let expected = portal_overlays
        .rows()
        .iter()
        .map(UiMountedPaintCommandIdentity::portal_overlay)
        .chain(
            semantic_text
                .rows()
                .iter()
                .map(UiMountedPaintCommandIdentity::semantic_text),
        )
        .collect::<HashSet<_>>();
    let observed = commands
        .iter()
        .map(|(identity, command)| {
            let derived = match command {
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                    UiMountedPaintCommandIdentity::portal_overlay(mechanic)
                }
                UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                    UiMountedPaintCommandIdentity::semantic_text(mechanic)
                }
            };
            (*identity == derived).then_some(*identity)
        })
        .collect::<Option<HashSet<_>>>()
        .ok_or(UiMountedPresentationReconstructionDenial::CommandMismatch)?;
    (expected == observed)
        .then_some(())
        .ok_or(UiMountedPresentationReconstructionDenial::CommandMismatch)
}

fn same_nodes(
    left: &[crate::UiMountedNodeProjectionView],
    right: &[crate::UiMountedNodeProjectionView],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.mounted_instance() == right.mounted_instance()
                && left.authored_position() == right.authored_position()
                && left.role() == right.role()
                && left.participation() == right.participation()
                && left.allocation() == right.allocation()
                && left.preview() == right.preview()
                && left.paint() == right.paint()
                && left.hit_test() == right.hit_test()
                && left.accessibility() == right.accessibility()
                && left.motion() == right.motion()
                && left.diagnostic() == right.diagnostic()
                && left.drawables() == right.drawables()
                && left.semantic_text() == right.semantic_text()
                && left.portal_presentation() == right.portal_presentation()
        })
}

fn same_hit_tests(
    left: &crate::UiMountedHitTestTable,
    right: &crate::UiMountedHitTestTable,
) -> bool {
    left.rows().len() == right.rows().len()
        && left.rows().iter().zip(right.rows()).all(|(left, right)| {
            left.surface() == right.surface()
                && left.binding() == right.binding()
                && left.mounted_instance() == right.mounted_instance()
                && left.bounds() == right.bounds()
                && left.clip_bounds() == right.clip_bounds()
                && left.order() == right.order()
        })
}
