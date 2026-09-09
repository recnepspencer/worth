use crate::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedStaticPaintSchemaVersion(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedRgba8([u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedFilledRectReference(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFilledRectCompletionDenial {
    NonAreaGeometry,
    ClipMismatch,
    NodeReceiptFrameMismatch,
    NodeReceiptInstanceMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFilledRectTableDenial {
    CapacityExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedFilledRectMechanic {
    schema: UiMountedStaticPaintSchemaVersion,
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    allocation_basis: super::UiMountedAllocationBasis,
    bounds: super::UiMountedCanonicalBox,
    color: UiMountedRgba8,
    layer_semantic_order: u32,
    clip_bounds: super::UiMountedCanonicalBox,
    semantic_digest: u64,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedFilledRectCompletionInput {
    pub frame: UiMountedFrameIdentity,
    pub surface: UiSemanticSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub mounted_instance: UiMountedInstanceIdentity,
    pub node_receipt: UiMountedNodeReceiptIdentity,
    pub allocation_basis: super::UiMountedAllocationBasis,
    pub bounds: super::UiMountedCanonicalBox,
    pub color: UiMountedRgba8,
    pub layer_semantic_order: u32,
    pub clip_bounds: super::UiMountedCanonicalBox,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedFilledRectTable {
    schema: UiMountedStaticPaintSchemaVersion,
    rows: std::sync::Arc<[UiMountedFilledRectMechanic]>,
}

impl UiMountedStaticPaintSchemaVersion {
    pub const REQUIRED_MOUNTED_FRAME_REVISION: u16 = 3;

    pub const fn current() -> Self {
        Self(1)
    }

    pub const fn revision(self) -> u16 {
        self.0
    }
}

impl UiMountedRgba8 {
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([red, green, blue, alpha])
    }

    pub const fn channels(self) -> [u8; 4] {
        self.0
    }
}

impl UiMountedFilledRectReference {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(index: u16) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u16 {
        self.0
    }
}

impl UiMountedFilledRectMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedFilledRectCompletionInput,
    ) -> Result<Self, UiMountedFilledRectCompletionDenial> {
        if input.bounds.posture() != super::UiMountedGeometryPosture::Area {
            return Err(UiMountedFilledRectCompletionDenial::NonAreaGeometry);
        }
        if input.clip_bounds.coordinate_space() != input.bounds.coordinate_space() {
            return Err(UiMountedFilledRectCompletionDenial::ClipMismatch);
        }
        if input.node_receipt.frame() != input.frame {
            return Err(UiMountedFilledRectCompletionDenial::NodeReceiptFrameMismatch);
        }
        if input.node_receipt.mounted_instance() != input.mounted_instance {
            return Err(UiMountedFilledRectCompletionDenial::NodeReceiptInstanceMismatch);
        }
        let semantic_digest = semantic_digest(&input);
        Ok(Self {
            schema: UiMountedStaticPaintSchemaVersion::current(),
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            mounted_instance: input.mounted_instance,
            node_receipt: input.node_receipt,
            allocation_basis: input.allocation_basis,
            bounds: input.bounds,
            color: input.color,
            layer_semantic_order: input.layer_semantic_order,
            clip_bounds: input.clip_bounds,
            semantic_digest,
        })
    }

    pub const fn schema(self) -> UiMountedStaticPaintSchemaVersion {
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

    pub const fn mounted_instance(self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub const fn node_receipt(self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub const fn allocation_basis(self) -> super::UiMountedAllocationBasis {
        self.allocation_basis
    }

    pub const fn bounds(self) -> super::UiMountedCanonicalBox {
        self.bounds
    }

    pub const fn color(self) -> UiMountedRgba8 {
        self.color
    }

    pub const fn layer_semantic_order(self) -> u32 {
        self.layer_semantic_order
    }

    pub const fn clip_bounds(self) -> super::UiMountedCanonicalBox {
        self.clip_bounds
    }

    pub const fn semantic_digest(self) -> u64 {
        self.semantic_digest
    }

    #[doc(hidden)]
    pub fn presented_within_portal(
        self,
        portal: super::UiMountedPortalOverlayMechanic,
    ) -> Result<Option<Self>, UiMountedFilledRectCompletionDenial> {
        let Some(geometry) =
            super::portal_child_geometry::project(self.bounds, self.clip_bounds, portal)
                .map_err(|_| UiMountedFilledRectCompletionDenial::NonAreaGeometry)?
        else {
            return Ok(None);
        };
        Self::complete_from_runtime_mounting(UiMountedFilledRectCompletionInput {
            frame: self.frame,
            surface: self.surface,
            binding: self.binding,
            mounted_instance: self.mounted_instance,
            node_receipt: self.node_receipt,
            allocation_basis: self.allocation_basis,
            bounds: geometry.bounds,
            color: self.color,
            layer_semantic_order: portal
                .layer_semantic_order()
                .saturating_add(1 + self.layer_semantic_order.min(1_024)),
            clip_bounds: geometry.clip,
        })
        .map(Some)
    }

    #[doc(hidden)]
    pub fn same_retained_paint_meaning(self, other: Self) -> bool {
        self.schema == other.schema
            && self.surface == other.surface
            && self.binding == other.binding
            && self.mounted_instance == other.mounted_instance
            && self.allocation_basis == other.allocation_basis
            && self.bounds == other.bounds
            && self.color == other.color
            && self.layer_semantic_order == other.layer_semantic_order
            && self.clip_bounds == other.clip_bounds
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

impl UiMountedFilledRectTable {
    pub const MAX_ROWS: usize = 2_048;

    pub fn empty() -> Self {
        Self {
            schema: UiMountedStaticPaintSchemaVersion::current(),
            rows: std::sync::Arc::from([]),
        }
    }

    #[doc(hidden)]
    pub fn from_runtime_mounting(
        rows: Vec<UiMountedFilledRectMechanic>,
    ) -> Result<Self, UiMountedFilledRectTableDenial> {
        if rows.len() > Self::MAX_ROWS {
            return Err(UiMountedFilledRectTableDenial::CapacityExceeded);
        }
        Ok(Self {
            schema: UiMountedStaticPaintSchemaVersion::current(),
            rows: rows.into(),
        })
    }

    pub const fn schema(&self) -> UiMountedStaticPaintSchemaVersion {
        self.schema
    }

    pub fn rows(&self) -> &[UiMountedFilledRectMechanic] {
        &self.rows
    }

    pub fn resolve(
        &self,
        reference: UiMountedFilledRectReference,
    ) -> Option<&UiMountedFilledRectMechanic> {
        self.rows.get(usize::from(reference.index()))
    }
}

fn semantic_digest(input: &UiMountedFilledRectCompletionInput) -> u64 {
    let mut digest = 0x6669_6c6c_7265_6374_u64;
    for value in [
        u64::from(UiMountedStaticPaintSchemaVersion::current().revision()),
        input.frame.diagnostic_value(),
        input.surface.diagnostic_value(),
        input.binding.diagnostic_value(),
        input.mounted_instance.diagnostic_value(),
        input.node_receipt.diagnostic_value(),
        input.allocation_basis.receipt_identity(),
        input.allocation_basis.receipt_generation(),
        input.allocation_basis.coordinate_ownership(),
        u64::from(input.bounds.x().to_bits()),
        u64::from(input.bounds.y().to_bits()),
        u64::from(input.bounds.width().to_bits()),
        u64::from(input.bounds.height().to_bits()),
        coordinate_space_digest(input.bounds.coordinate_space()),
        geometry_posture_digest(input.bounds.posture()),
        u64::from(input.clip_bounds.x().to_bits()),
        u64::from(input.clip_bounds.y().to_bits()),
        u64::from(input.clip_bounds.width().to_bits()),
        u64::from(input.clip_bounds.height().to_bits()),
        coordinate_space_digest(input.clip_bounds.coordinate_space()),
        geometry_posture_digest(input.clip_bounds.posture()),
        transform_digest(input.allocation_basis.transform()),
        u64::from(input.layer_semantic_order),
    ] {
        digest = fold(digest, value);
    }
    for channel in input.color.channels() {
        digest = fold(digest, u64::from(channel));
    }
    digest
}

fn coordinate_space_digest(space: super::UiMountedCoordinateSpace) -> u64 {
    match space {
        super::UiMountedCoordinateSpace::Viewport => 1,
        super::UiMountedCoordinateSpace::Window => 2,
        super::UiMountedCoordinateSpace::GraphNodeLocal => 3,
        super::UiMountedCoordinateSpace::HostSurface => 4,
        super::UiMountedCoordinateSpace::PortalLayer => 5,
    }
}

fn geometry_posture_digest(posture: super::UiMountedGeometryPosture) -> u64 {
    match posture {
        super::UiMountedGeometryPosture::Area => 1,
        super::UiMountedGeometryPosture::Empty => 2,
        super::UiMountedGeometryPosture::Offscreen => 3,
    }
}

fn transform_digest(transform: super::UiMountedTransformProjection) -> u64 {
    match transform {
        super::UiMountedTransformProjection::Identity => 1,
        super::UiMountedTransformProjection::Omitted(reason) => {
            2 ^ omission_reason_digest(reason).rotate_left(11)
        }
    }
}

fn omission_reason_digest(reason: super::UiMountedOmissionReason) -> u64 {
    match reason {
        super::UiMountedOmissionReason::AwaitingRuntimeMutation => 1,
        super::UiMountedOmissionReason::NotDefinedByCurrentRuntime => 2,
        super::UiMountedOmissionReason::NoCommittedAllocation => 3,
        super::UiMountedOmissionReason::AllocationBoundsUnknown => 4,
        super::UiMountedOmissionReason::SurfacePolicyWithheld => 5,
        super::UiMountedOmissionReason::NotProducedByExecutedLane => 6,
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(0x100000001b3)
}

#[cfg(test)]
#[path = "static_paint_tests.rs"]
mod tests;
