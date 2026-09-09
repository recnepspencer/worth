use crate::{
    UiHostObservationPresentationBasis, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedPortalOverlaySchemaVersion(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedPortalOverlayReference(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPortalOverlayLifecyclePosture {
    Visible,
    Closing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPortalInputShielding {
    ContentBounds,
    ModalSurface,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPortalOverlayCompletionDenial {
    NonAreaGeometry,
    CoordinateSpaceMismatch,
    NodeReceiptFrameMismatch,
    NodeReceiptInstanceMismatch,
    LayerOrderOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedPortalOverlayMechanic {
    schema: UiMountedPortalOverlaySchemaVersion,
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    owner: UiMountedInstanceIdentity,
    owner_receipt: UiMountedNodeReceiptIdentity,
    portal_identity: u64,
    anchor_presentation: UiHostObservationPresentationBasis,
    anchor_bounds: super::UiMountedCanonicalBox,
    bounds: super::UiMountedCanonicalBox,
    clip_bounds: super::UiMountedCanonicalBox,
    color: super::UiMountedRgba8,
    layer_semantic_order: u32,
    layer_depth: u16,
    lifecycle: UiMountedPortalOverlayLifecyclePosture,
    shielding: UiMountedPortalInputShielding,
    semantic_digest: u64,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedPortalOverlayCompletionInput {
    pub frame: UiMountedFrameIdentity,
    pub surface: UiSemanticSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub owner: UiMountedInstanceIdentity,
    pub owner_receipt: UiMountedNodeReceiptIdentity,
    pub portal_identity: u64,
    pub anchor_presentation: UiHostObservationPresentationBasis,
    pub anchor_bounds: super::UiMountedCanonicalBox,
    pub bounds: super::UiMountedCanonicalBox,
    pub clip_bounds: super::UiMountedCanonicalBox,
    pub color: super::UiMountedRgba8,
    pub layer_semantic_order: u32,
    pub layer_depth: u16,
    pub lifecycle: UiMountedPortalOverlayLifecyclePosture,
    pub shielding: UiMountedPortalInputShielding,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedPortalOverlayTable {
    schema: UiMountedPortalOverlaySchemaVersion,
    rows: std::sync::Arc<[UiMountedPortalOverlayMechanic]>,
}

impl Eq for UiMountedPortalOverlayMechanic {}
impl Eq for UiMountedPortalOverlayCompletionInput {}

impl UiMountedPortalOverlaySchemaVersion {
    pub const REQUIRED_MOUNTED_FRAME_REVISION: u16 = 3;

    pub const fn current() -> Self {
        Self(1)
    }

    pub const fn revision(self) -> u16 {
        self.0
    }
}

impl UiMountedPortalOverlayReference {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(index: u16) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u16 {
        self.0
    }
}

impl UiMountedPortalOverlayMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedPortalOverlayCompletionInput,
    ) -> Result<Self, UiMountedPortalOverlayCompletionDenial> {
        if input.bounds.posture() != super::UiMountedGeometryPosture::Area
            || input.anchor_bounds.posture() != super::UiMountedGeometryPosture::Area
            || input.clip_bounds.posture() != super::UiMountedGeometryPosture::Area
        {
            return Err(UiMountedPortalOverlayCompletionDenial::NonAreaGeometry);
        }
        if input.bounds.coordinate_space() != super::UiMountedCoordinateSpace::Viewport
            || input.anchor_bounds.coordinate_space() != super::UiMountedCoordinateSpace::Viewport
            || input.clip_bounds.coordinate_space() != super::UiMountedCoordinateSpace::Viewport
        {
            return Err(UiMountedPortalOverlayCompletionDenial::CoordinateSpaceMismatch);
        }
        if input.owner_receipt.frame() != input.frame {
            return Err(UiMountedPortalOverlayCompletionDenial::NodeReceiptFrameMismatch);
        }
        if input.owner_receipt.mounted_instance() != input.owner {
            return Err(UiMountedPortalOverlayCompletionDenial::NodeReceiptInstanceMismatch);
        }
        if u32::from(input.layer_depth) > input.layer_semantic_order {
            return Err(UiMountedPortalOverlayCompletionDenial::LayerOrderOverflow);
        }
        let semantic_digest = semantic_digest(&input);
        Ok(Self {
            schema: UiMountedPortalOverlaySchemaVersion::current(),
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            owner: input.owner,
            owner_receipt: input.owner_receipt,
            portal_identity: input.portal_identity,
            anchor_presentation: input.anchor_presentation,
            anchor_bounds: input.anchor_bounds,
            bounds: input.bounds,
            clip_bounds: input.clip_bounds,
            color: input.color,
            layer_semantic_order: input.layer_semantic_order,
            layer_depth: input.layer_depth,
            lifecycle: input.lifecycle,
            shielding: input.shielding,
            semantic_digest,
        })
    }

    pub const fn schema(self) -> UiMountedPortalOverlaySchemaVersion {
        self.schema
    }
    pub const fn frame(self) -> UiMountedFrameIdentity {
        self.frame
    }
    pub const fn surface(self) -> UiSemanticSurfaceIdentity {
        self.surface
    }
    pub const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }
    pub const fn owner(self) -> UiMountedInstanceIdentity {
        self.owner
    }
    pub const fn owner_receipt(self) -> UiMountedNodeReceiptIdentity {
        self.owner_receipt
    }
    pub const fn portal_identity(self) -> u64 {
        self.portal_identity
    }
    pub const fn anchor_presentation(self) -> UiHostObservationPresentationBasis {
        self.anchor_presentation
    }
    pub const fn anchor_bounds(self) -> super::UiMountedCanonicalBox {
        self.anchor_bounds
    }
    pub const fn bounds(self) -> super::UiMountedCanonicalBox {
        self.bounds
    }
    pub const fn clip_bounds(self) -> super::UiMountedCanonicalBox {
        self.clip_bounds
    }
    pub const fn color(self) -> super::UiMountedRgba8 {
        self.color
    }
    pub const fn layer_semantic_order(self) -> u32 {
        self.layer_semantic_order
    }
    pub const fn layer_depth(self) -> u16 {
        self.layer_depth
    }
    pub const fn lifecycle(self) -> UiMountedPortalOverlayLifecyclePosture {
        self.lifecycle
    }
    pub const fn shielding(self) -> UiMountedPortalInputShielding {
        self.shielding
    }
    pub const fn semantic_digest(self) -> u64 {
        self.semantic_digest
    }

    #[doc(hidden)]
    pub fn same_retained_paint_meaning(self, other: Self) -> bool {
        self.schema == other.schema
            && self.surface == other.surface
            && self.binding == other.binding
            && self.owner == other.owner
            && self.portal_identity == other.portal_identity
            && self.anchor_presentation.host_surface() == other.anchor_presentation.host_surface()
            && self.anchor_presentation.binding() == other.anchor_presentation.binding()
            && self.anchor_bounds == other.anchor_bounds
            && self.bounds == other.bounds
            && self.clip_bounds == other.clip_bounds
            && self.color == other.color
            && self.layer_semantic_order == other.layer_semantic_order
            && self.layer_depth == other.layer_depth
            && self.lifecycle == other.lifecycle
            && self.shielding == other.shielding
    }

    /// Reconciliation may move one unchanged command onto the admitted replacement binding.
    #[doc(hidden)]
    pub fn same_retained_paint_meaning_after_binding_replacement(
        self,
        other: Self,
        affected: UiSurfaceBindingGeneration,
        replacement: UiSurfaceBindingGeneration,
    ) -> bool {
        if self.binding != affected || other.binding != replacement {
            return false;
        }
        let mut rebound = self;
        rebound.binding = replacement;
        rebound.same_retained_paint_meaning(other)
    }
}

impl UiMountedPortalOverlayTable {
    pub const MAX_ROWS: usize = 1_024;

    pub fn empty() -> Self {
        Self {
            schema: UiMountedPortalOverlaySchemaVersion::current(),
            rows: std::sync::Arc::from([]),
        }
    }

    #[doc(hidden)]
    pub fn from_runtime_mounting(rows: Vec<UiMountedPortalOverlayMechanic>) -> Option<Self> {
        (rows.len() <= Self::MAX_ROWS).then(|| Self {
            schema: UiMountedPortalOverlaySchemaVersion::current(),
            rows: rows.into(),
        })
    }

    pub const fn schema(&self) -> UiMountedPortalOverlaySchemaVersion {
        self.schema
    }
    pub fn rows(&self) -> &[UiMountedPortalOverlayMechanic] {
        &self.rows
    }
    pub fn resolve(
        &self,
        reference: UiMountedPortalOverlayReference,
    ) -> Option<&UiMountedPortalOverlayMechanic> {
        self.rows.get(usize::from(reference.index()))
    }
}

fn semantic_digest(input: &UiMountedPortalOverlayCompletionInput) -> u64 {
    let mut digest = 0x706f_7274_616c_6f76_u64;
    let bounds = [input.anchor_bounds, input.bounds, input.clip_bounds];
    for value in [
        input.frame.diagnostic_value(),
        input.surface.diagnostic_value(),
        input.binding.diagnostic_value(),
        input.owner.diagnostic_value(),
        input.owner_receipt.diagnostic_value(),
        input.portal_identity,
        input.anchor_presentation.frame().diagnostic_value(),
        input.anchor_presentation.binding().diagnostic_value(),
        input.anchor_presentation.epoch().diagnostic_value(),
        u64::from(input.layer_semantic_order),
        u64::from(input.layer_depth),
        input.lifecycle as u64,
        input.shielding as u64,
    ] {
        digest = (digest ^ value).wrapping_mul(0x100000001b3);
    }
    for bounds in bounds {
        for value in [bounds.x(), bounds.y(), bounds.width(), bounds.height()] {
            digest = (digest ^ u64::from(value.to_bits())).wrapping_mul(0x100000001b3);
        }
    }
    for channel in input.color.channels() {
        digest = (digest ^ u64::from(channel)).wrapping_mul(0x100000001b3);
    }
    digest
}
