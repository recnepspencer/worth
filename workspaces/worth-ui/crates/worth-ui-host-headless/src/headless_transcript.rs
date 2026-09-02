use worth_ui_host_contract::{
    UiHostProtocolAgreement, UiHostSurfacePresentationMode, UiMountedAccessibilityProjection,
    UiMountedAllocationProjection, UiMountedCanonicalBox, UiMountedEffectFamily,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedLogicalDamage,
    UiMountedMechanicalRole, UiMountedMotionProjection, UiMountedOmissionReason,
    UiMountedPaintOrderIdentity, UiMountedPaintPrimitiveKind, UiMountedParticipation,
    UiMountedPresentationAttemptIdentity, UiMountedPreviewProjection, UiMountedResourceKind,
    UiSurfaceBindingGeneration,
};

#[cfg(test)]
pub(crate) mod appearance;
mod delta;
mod mechanic_accessors;
mod semantic_text;
mod static_paint;
mod text_accessibility;
mod text_measurement;

pub use self::semantic_text::UiHeadlessSemanticTextMechanic;
pub(crate) use self::semantic_text::UiHeadlessSemanticTextMechanicInput;
pub use self::static_paint::UiHeadlessFilledRectMechanic;
pub(crate) use self::static_paint::UiHeadlessFilledRectMechanicInput;
pub use self::text_accessibility::UiHeadlessTextAccessibilityGeometry;
pub use self::text_measurement::UiHeadlessTextMeasurement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHeadlessRecorderCapacity {
    surface_bindings: usize,
    retained_frames: usize,
    mechanics_per_frame: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiHeadlessResolvedClip {
    Unclipped,
    Clip(u16),
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiHeadlessLayerMechanic {
    Ordered {
        semantic_order: u32,
        clip: UiHeadlessResolvedClip,
    },
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiHeadlessClipMechanic {
    bounds: UiMountedCanonicalBox,
    parent: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHeadlessResourceContact {
    content_identity: u64,
    kind: UiMountedResourceKind,
    byte_len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiHeadlessPaintBatchMechanic {
    batch_index: u16,
    primitive_kind: UiMountedPaintPrimitiveKind,
    primitive_count: u32,
    layer: UiHeadlessLayerMechanic,
    resource: Option<UiHeadlessResourceContact>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHeadlessNodePaintMechanic {
    CountOnlyBatch(u16),
    FilledRect(u16),
    Omitted(UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiHeadlessNodeMechanic {
    mounted_instance: UiMountedInstanceIdentity,
    authored_position: u64,
    role: UiMountedMechanicalRole,
    participation: UiMountedParticipation,
    allocation: UiMountedAllocationProjection,
    preview: UiMountedPreviewProjection,
    paint: UiHeadlessNodePaintMechanic,
    accessibility: UiMountedAccessibilityProjection,
    motion: UiMountedMotionProjection,
    diagnostic: worth_ui_host_contract::UiMountedDiagnosticProjection,
}

pub(crate) struct UiHeadlessNodeMechanicInput {
    pub mounted_instance: UiMountedInstanceIdentity,
    pub authored_position: u64,
    pub role: UiMountedMechanicalRole,
    pub participation: UiMountedParticipation,
    pub allocation: UiMountedAllocationProjection,
    pub preview: UiMountedPreviewProjection,
    pub paint: UiHeadlessNodePaintMechanic,
    pub accessibility: UiMountedAccessibilityProjection,
    pub motion: UiMountedMotionProjection,
    pub diagnostic: worth_ui_host_contract::UiMountedDiagnosticProjection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHeadlessUnperformedEffect {
    NativePaint {
        filled_rect_count: u32,
        portal_overlay_count: u32,
        semantic_text_count: u32,
        preview_node_count: u32,
    },
    Accessibility {
        node_count: u32,
    },
    Diagnostic {
        node_count: u32,
    },
    CanvasSpatial {
        batch_index: u16,
        primitive_count: u32,
        hit_region_count: u32,
        overlay_row_count: u16,
        tool_state_row_count: u16,
    },
    Realtime {
        batch_index: u16,
        overlay_row_count: u16,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHeadlessMountedFrameTranscript {
    host_session_identity: u64,
    protocol: UiHostProtocolAgreement,
    attempt: UiMountedPresentationAttemptIdentity,
    frame: UiMountedFrameIdentity,
    binding: UiSurfaceBindingGeneration,
    mode: UiHostSurfacePresentationMode,
    nodes: Box<[UiHeadlessNodeMechanic]>,
    clips: Box<[UiHeadlessClipMechanic]>,
    filled_rects: Box<[UiHeadlessFilledRectMechanic]>,
    portal_overlays: Box<[worth_ui_host_contract::UiMountedPortalOverlayMechanic]>,
    semantic_text: Box<[UiHeadlessSemanticTextMechanic]>,
    paint_batches: Box<[UiHeadlessPaintBatchMechanic]>,
    paint_order: Box<[UiMountedPaintOrderIdentity]>,
    logical_damage: Box<[UiMountedLogicalDamage]>,
    unperformed_effects: Box<[UiHeadlessUnperformedEffect]>,
}

pub(crate) struct UiHeadlessMountedFrameTranscriptInput {
    pub host_session_identity: u64,
    pub protocol: UiHostProtocolAgreement,
    pub attempt: UiMountedPresentationAttemptIdentity,
    pub frame: UiMountedFrameIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub nodes: Vec<UiHeadlessNodeMechanic>,
    pub clips: Vec<UiHeadlessClipMechanic>,
    pub filled_rects: Vec<UiHeadlessFilledRectMechanic>,
    pub portal_overlays: Vec<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    pub semantic_text: Vec<UiHeadlessSemanticTextMechanic>,
    pub paint_batches: Vec<UiHeadlessPaintBatchMechanic>,
    pub paint_order: Vec<UiMountedPaintOrderIdentity>,
    pub logical_damage: Vec<UiMountedLogicalDamage>,
    pub unperformed_effects: Vec<UiHeadlessUnperformedEffect>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiHeadlessTranscriptSuccessorIdentity {
    pub host_session_identity: u64,
    pub protocol: UiHostProtocolAgreement,
    pub attempt: UiMountedPresentationAttemptIdentity,
    pub frame: UiMountedFrameIdentity,
    pub binding: UiSurfaceBindingGeneration,
}

impl UiHeadlessMountedFrameTranscript {
    pub(crate) fn new(input: UiHeadlessMountedFrameTranscriptInput) -> Self {
        Self {
            host_session_identity: input.host_session_identity,
            protocol: input.protocol,
            attempt: input.attempt,
            frame: input.frame,
            binding: input.binding,
            mode: UiHostSurfacePresentationMode::RecordOnly,
            nodes: input.nodes.into_boxed_slice(),
            clips: input.clips.into_boxed_slice(),
            filled_rects: input.filled_rects.into_boxed_slice(),
            portal_overlays: input.portal_overlays.into_boxed_slice(),
            semantic_text: input.semantic_text.into_boxed_slice(),
            paint_batches: input.paint_batches.into_boxed_slice(),
            paint_order: input.paint_order.into_boxed_slice(),
            logical_damage: input.logical_damage.into_boxed_slice(),
            unperformed_effects: input.unperformed_effects.into_boxed_slice(),
        }
    }

    pub const fn host_session_identity(&self) -> u64 {
        self.host_session_identity
    }

    pub const fn protocol(&self) -> UiHostProtocolAgreement {
        self.protocol
    }

    pub const fn attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub const fn frame(&self) -> UiMountedFrameIdentity {
        self.frame
    }

    pub const fn binding(&self) -> UiSurfaceBindingGeneration {
        self.binding
    }

    pub const fn mode(&self) -> UiHostSurfacePresentationMode {
        self.mode
    }

    pub fn nodes(&self) -> &[UiHeadlessNodeMechanic] {
        &self.nodes
    }

    pub fn clips(&self) -> &[UiHeadlessClipMechanic] {
        &self.clips
    }

    pub fn filled_rects(&self) -> &[UiHeadlessFilledRectMechanic] {
        &self.filled_rects
    }

    pub fn portal_overlays(&self) -> &[worth_ui_host_contract::UiMountedPortalOverlayMechanic] {
        &self.portal_overlays
    }

    pub fn semantic_text(&self) -> &[UiHeadlessSemanticTextMechanic] {
        &self.semantic_text
    }

    pub fn paint_batches(&self) -> &[UiHeadlessPaintBatchMechanic] {
        &self.paint_batches
    }

    pub fn paint_order(&self) -> &[UiMountedPaintOrderIdentity] {
        &self.paint_order
    }

    pub fn logical_damage(&self) -> &[UiMountedLogicalDamage] {
        &self.logical_damage
    }

    pub fn unperformed_effects(&self) -> &[UiHeadlessUnperformedEffect] {
        &self.unperformed_effects
    }
}
