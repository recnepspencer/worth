use crate::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};
use std::collections::HashMap;
use std::sync::Arc;

#[path = "view/presentation_effects.rs"]
mod presentation_effects;
#[path = "view/presentation_sources.rs"]
mod presentation_sources;

use presentation_sources::PresentationSources;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedDrawableReference {
    PortalOverlay(super::UiMountedPortalOverlayReference),
    SemanticText(super::UiMountedSemanticTextReference),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedNodeProjectionView {
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    authored_position: u64,
    role: super::UiMountedMechanicalRole,
    participation: super::UiMountedParticipation,
    allocation: super::UiMountedAllocationProjection,
    preview: super::UiMountedPreviewProjection,
    paint: super::UiMountedPaintProjection,
    hit_test: super::UiMountedHitTestProjection,
    accessibility: super::UiMountedAccessibilityProjection,
    motion: super::UiMountedMotionProjection,
    diagnostic: super::UiMountedDiagnosticProjection,
    drawables: Box<[UiMountedDrawableReference]>,
    semantic_text: Box<[super::UiMountedSemanticTextReference]>,
    portal_presentation: Option<super::UiMountedPortalPresentationAffinity>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedProjectionView {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    content_generation: crate::UiMountedContentGeneration,
    nodes: Arc<[UiMountedNodeProjectionView]>,
    clips: super::UiMountedClipTable,
    layers: super::UiMountedLayerTable,
    portal_overlays: super::UiMountedPortalOverlayTable,
    semantic_text: super::UiMountedSemanticTextTable,
    hit_tests: super::UiMountedHitTestTable,
    paint_batches: super::UiMountedPaintBatchTable,
    spatial_batches: super::UiMountedSpatialBatchTable,
    realtime_batches: super::UiMountedRealtimeBatchTable,
    resources: super::UiMountedResourceTable,
    paint_commands: Arc<[crate::UiMountedPaintCommand]>,
    paint_order: Arc<[crate::UiMountedPaintOrderIdentity]>,
    command_indices: Arc<HashMap<crate::UiMountedPaintCommandIdentity, usize>>,
    commands_by_instance:
        Arc<HashMap<UiMountedInstanceIdentity, Arc<[crate::UiMountedPaintCommandIdentity]>>>,
    order_predecessors: Arc<
        HashMap<crate::UiMountedPaintCommandIdentity, Option<crate::UiMountedPaintOrderIdentity>>,
    >,
    order_positions: Arc<HashMap<crate::UiMountedPaintCommandIdentity, usize>>,
    order_integrity: crate::UiMountedPaintOrderIntegrity,
    native_effects: Arc<[crate::UiMountedEffectFamily]>,
}

pub struct UiMountedNodeProjectionViewInput {
    pub mounted_instance: UiMountedInstanceIdentity,
    pub node_receipt: UiMountedNodeReceiptIdentity,
    pub authored_position: u64,
    pub role: super::UiMountedMechanicalRole,
    pub participation: super::UiMountedParticipation,
    pub allocation: super::UiMountedAllocationProjection,
    pub preview: super::UiMountedPreviewProjection,
    pub paint: super::UiMountedPaintProjection,
    pub hit_test: super::UiMountedHitTestProjection,
    pub accessibility: super::UiMountedAccessibilityProjection,
    pub motion: super::UiMountedMotionProjection,
    pub diagnostic: super::UiMountedDiagnosticProjection,
    pub drawables: Vec<UiMountedDrawableReference>,
    pub semantic_text: Vec<super::UiMountedSemanticTextReference>,
    pub portal_presentation: Option<super::UiMountedPortalPresentationAffinity>,
}

pub struct UiMountedProjectionViewInput {
    pub frame: UiMountedFrameIdentity,
    pub surface: UiSemanticSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub content_generation: crate::UiMountedContentGeneration,
    pub nodes: Vec<UiMountedNodeProjectionView>,
    pub clips: super::UiMountedClipTable,
    pub layers: super::UiMountedLayerTable,
    pub portal_overlays: super::UiMountedPortalOverlayTable,
    pub semantic_text: super::UiMountedSemanticTextTable,
    pub hit_tests: super::UiMountedHitTestTable,
    pub paint_batches: super::UiMountedPaintBatchTable,
    pub spatial_batches: super::UiMountedSpatialBatchTable,
    pub realtime_batches: super::UiMountedRealtimeBatchTable,
    pub resources: super::UiMountedResourceTable,
    pub authored_paint_commands: Vec<crate::UiMountedPaintCommand>,
    pub authored_paint_order: Vec<crate::UiMountedPaintOrderIdentity>,
}

impl UiMountedNodeProjectionView {
    pub fn new(input: UiMountedNodeProjectionViewInput) -> Self {
        Self {
            mounted_instance: input.mounted_instance,
            node_receipt: input.node_receipt,
            authored_position: input.authored_position,
            role: input.role,
            participation: input.participation,
            allocation: input.allocation,
            preview: input.preview,
            paint: input.paint,
            hit_test: input.hit_test,
            accessibility: input.accessibility,
            motion: input.motion,
            diagnostic: input.diagnostic,
            drawables: input.drawables.into_boxed_slice(),
            semantic_text: input.semantic_text.into_boxed_slice(),
            portal_presentation: input.portal_presentation,
        }
    }
    pub fn mounted_instance(&self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub fn node_receipt(&self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn authored_position(&self) -> u64 {
        self.authored_position
    }
    pub fn role(&self) -> super::UiMountedMechanicalRole {
        self.role
    }
    pub fn participation(&self) -> super::UiMountedParticipation {
        self.participation
    }
    pub fn allocation(&self) -> super::UiMountedAllocationProjection {
        self.allocation
    }
    pub fn preview(&self) -> super::UiMountedPreviewProjection {
        self.preview
    }
    pub fn paint(&self) -> super::UiMountedPaintProjection {
        self.paint
    }
    pub fn hit_test(&self) -> super::UiMountedHitTestProjection {
        self.hit_test
    }
    pub fn accessibility(&self) -> super::UiMountedAccessibilityProjection {
        self.accessibility
    }
    pub fn motion(&self) -> super::UiMountedMotionProjection {
        self.motion
    }
    pub fn diagnostic(&self) -> super::UiMountedDiagnosticProjection {
        self.diagnostic
    }
    pub fn drawables(&self) -> &[UiMountedDrawableReference] {
        &self.drawables
    }
    pub fn semantic_text(&self) -> &[super::UiMountedSemanticTextReference] {
        &self.semantic_text
    }
    pub const fn portal_presentation(&self) -> Option<super::UiMountedPortalPresentationAffinity> {
        self.portal_presentation
    }
}

impl UiMountedProjectionView {
    #[doc(hidden)]
    pub fn node_receipt_affinity(&self) -> Option<crate::UiMountedNodeReceiptAffinity> {
        self.nodes
            .first()
            .map(|node| crate::UiMountedNodeReceiptAffinity::from_receipt(node.node_receipt()))
    }

    pub fn new(input: UiMountedProjectionViewInput) -> Self {
        let native_effects = presentation_effects::derive(
            &input.nodes,
            &input.paint_batches,
            &input.spatial_batches,
            &input.realtime_batches,
        );
        let presentation = PresentationSources::admit(
            &input.nodes,
            &input.portal_overlays,
            &input.semantic_text,
            input.authored_paint_commands,
            input.authored_paint_order,
        );
        Self {
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            content_generation: input.content_generation,
            nodes: input.nodes.into(),
            clips: input.clips,
            layers: input.layers,
            portal_overlays: input.portal_overlays,
            semantic_text: input.semantic_text,
            hit_tests: input.hit_tests,
            paint_batches: input.paint_batches,
            spatial_batches: input.spatial_batches,
            realtime_batches: input.realtime_batches,
            resources: input.resources,
            paint_commands: presentation.commands.into(),
            paint_order: presentation.order.into(),
            command_indices: Arc::new(presentation.command_indices),
            commands_by_instance: Arc::new(presentation.commands_by_instance),
            order_predecessors: Arc::new(presentation.order_predecessors),
            order_positions: Arc::new(presentation.order_positions),
            order_integrity: presentation.order_integrity,
            native_effects: native_effects.into(),
        }
    }
    pub fn frame(&self) -> UiMountedFrameIdentity {
        self.frame
    }
    pub fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }
    pub fn binding(&self) -> UiSurfaceBindingGeneration {
        self.binding
    }
    pub fn content_generation(&self) -> crate::UiMountedContentGeneration {
        self.content_generation
    }
    pub fn nodes(&self) -> &[UiMountedNodeProjectionView] {
        &self.nodes
    }

    #[doc(hidden)]
    pub fn retained_nodes(&self) -> Arc<[UiMountedNodeProjectionView]> {
        Arc::clone(&self.nodes)
    }
    pub fn clips(&self) -> &super::UiMountedClipTable {
        &self.clips
    }
    pub fn layers(&self) -> &super::UiMountedLayerTable {
        &self.layers
    }
    pub fn portal_overlays(&self) -> &super::UiMountedPortalOverlayTable {
        &self.portal_overlays
    }
    pub fn semantic_text(&self) -> &super::UiMountedSemanticTextTable {
        &self.semantic_text
    }
    pub fn hit_tests(&self) -> &super::UiMountedHitTestTable {
        &self.hit_tests
    }
    pub fn paint_batches(&self) -> &super::UiMountedPaintBatchTable {
        &self.paint_batches
    }
    pub fn spatial_batches(&self) -> &super::UiMountedSpatialBatchTable {
        &self.spatial_batches
    }
    pub fn realtime_batches(&self) -> &super::UiMountedRealtimeBatchTable {
        &self.realtime_batches
    }
    pub fn resources(&self) -> &super::UiMountedResourceTable {
        &self.resources
    }

    #[doc(hidden)]
    pub fn authored_paint_commands(&self) -> &[crate::UiMountedPaintCommand] {
        &self.paint_commands
    }

    #[doc(hidden)]
    pub fn retained_paint_commands(&self) -> Arc<[crate::UiMountedPaintCommand]> {
        Arc::clone(&self.paint_commands)
    }

    #[doc(hidden)]
    pub fn authored_paint_order(&self) -> &[crate::UiMountedPaintOrderIdentity] {
        &self.paint_order
    }

    #[doc(hidden)]
    pub fn retained_paint_order(&self) -> Arc<[crate::UiMountedPaintOrderIdentity]> {
        Arc::clone(&self.paint_order)
    }

    #[doc(hidden)]
    pub fn authored_paint_command(
        &self,
        identity: crate::UiMountedPaintCommandIdentity,
    ) -> Option<&crate::UiMountedPaintCommand> {
        self.command_indices
            .get(&identity)
            .and_then(|index| self.paint_commands.get(*index))
    }

    #[doc(hidden)]
    pub fn authored_commands_for_instance(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> &[crate::UiMountedPaintCommandIdentity] {
        self.commands_by_instance
            .get(&instance)
            .map_or(&[], AsRef::as_ref)
    }

    #[doc(hidden)]
    pub fn authored_paint_predecessor(
        &self,
        identity: crate::UiMountedPaintCommandIdentity,
    ) -> Option<Option<crate::UiMountedPaintOrderIdentity>> {
        self.order_predecessors.get(&identity).copied()
    }

    #[doc(hidden)]
    pub fn retained_command_indices(
        &self,
    ) -> Arc<HashMap<crate::UiMountedPaintCommandIdentity, usize>> {
        Arc::clone(&self.command_indices)
    }

    #[doc(hidden)]
    pub fn retained_commands_by_instance(
        &self,
    ) -> Arc<HashMap<UiMountedInstanceIdentity, Arc<[crate::UiMountedPaintCommandIdentity]>>> {
        Arc::clone(&self.commands_by_instance)
    }

    #[doc(hidden)]
    pub fn retained_order_predecessors(
        &self,
    ) -> Arc<
        HashMap<crate::UiMountedPaintCommandIdentity, Option<crate::UiMountedPaintOrderIdentity>>,
    > {
        Arc::clone(&self.order_predecessors)
    }

    #[doc(hidden)]
    pub fn retained_order_positions(
        &self,
    ) -> Arc<HashMap<crate::UiMountedPaintCommandIdentity, usize>> {
        Arc::clone(&self.order_positions)
    }

    #[doc(hidden)]
    pub const fn retained_order_integrity(&self) -> crate::UiMountedPaintOrderIntegrity {
        self.order_integrity
    }

    #[doc(hidden)]
    pub fn authored_native_effects(&self) -> &[crate::UiMountedEffectFamily] {
        &self.native_effects
    }
}
