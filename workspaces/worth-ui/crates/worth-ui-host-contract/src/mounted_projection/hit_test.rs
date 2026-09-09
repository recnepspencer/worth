use crate::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum UiMountedHitTestPresentationLayer {
    Portal,
    Ordinary,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiMountedHitTestOrder {
    presentation_layer: UiMountedHitTestPresentationLayer,
    rank: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedHitTestReference(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedHitTestProjection {
    Region(UiMountedHitTestReference),
    Omitted(super::UiMountedOmissionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedHitTestCompletionDenial {
    NonAreaGeometry,
    CoordinateSpaceMismatch,
    NodeReceiptFrameMismatch,
    NodeReceiptInstanceMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedHitTestMechanic {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    bounds: super::UiMountedCanonicalBox,
    clip_bounds: super::UiMountedCanonicalBox,
    order: UiMountedHitTestOrder,
    semantic_digest: u64,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedHitTestCompletionInput {
    pub frame: UiMountedFrameIdentity,
    pub surface: UiSemanticSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub mounted_instance: UiMountedInstanceIdentity,
    pub node_receipt: UiMountedNodeReceiptIdentity,
    pub bounds: super::UiMountedCanonicalBox,
    pub clip_bounds: super::UiMountedCanonicalBox,
    pub order: UiMountedHitTestOrder,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedHitTestTable {
    rows: std::sync::Arc<[UiMountedHitTestMechanic]>,
}

impl UiMountedHitTestOrder {
    #[doc(hidden)]
    pub const fn from_runtime_plan(rank: u32) -> Self {
        Self {
            presentation_layer: UiMountedHitTestPresentationLayer::Ordinary,
            rank,
        }
    }

    pub const fn rank(self) -> u32 {
        self.rank
    }

    const fn presented_within_portal(self) -> Self {
        Self {
            presentation_layer: UiMountedHitTestPresentationLayer::Portal,
            rank: self.rank,
        }
    }

    const fn presentation_layer_digest(self) -> u64 {
        match self.presentation_layer {
            UiMountedHitTestPresentationLayer::Portal => 1,
            UiMountedHitTestPresentationLayer::Ordinary => 2,
        }
    }
}

impl UiMountedHitTestReference {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u32 {
        self.0
    }
}

impl UiMountedHitTestMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedHitTestCompletionInput,
    ) -> Result<Self, UiMountedHitTestCompletionDenial> {
        if input.bounds.posture() != super::UiMountedGeometryPosture::Area
            || input.clip_bounds.posture() != super::UiMountedGeometryPosture::Area
        {
            return Err(UiMountedHitTestCompletionDenial::NonAreaGeometry);
        }
        if input.bounds.coordinate_space() != input.clip_bounds.coordinate_space() {
            return Err(UiMountedHitTestCompletionDenial::CoordinateSpaceMismatch);
        }
        if input.node_receipt.frame() != input.frame {
            return Err(UiMountedHitTestCompletionDenial::NodeReceiptFrameMismatch);
        }
        if input.node_receipt.mounted_instance() != input.mounted_instance {
            return Err(UiMountedHitTestCompletionDenial::NodeReceiptInstanceMismatch);
        }
        Ok(Self {
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            mounted_instance: input.mounted_instance,
            node_receipt: input.node_receipt,
            bounds: input.bounds,
            clip_bounds: input.clip_bounds,
            order: input.order,
            semantic_digest: semantic_digest(&input),
        })
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
    pub const fn bounds(self) -> super::UiMountedCanonicalBox {
        self.bounds
    }
    pub const fn clip_bounds(self) -> super::UiMountedCanonicalBox {
        self.clip_bounds
    }
    pub const fn order(self) -> UiMountedHitTestOrder {
        self.order
    }
    pub const fn semantic_digest(self) -> u64 {
        self.semantic_digest
    }

    #[doc(hidden)]
    pub fn presented_within_portal(
        self,
        portal: super::UiMountedPortalOverlayMechanic,
    ) -> Result<Option<Self>, UiMountedHitTestCompletionDenial> {
        let Some(geometry) =
            super::portal_child_geometry::project(self.bounds, self.clip_bounds, portal)
                .map_err(|_| UiMountedHitTestCompletionDenial::NonAreaGeometry)?
        else {
            return Ok(None);
        };
        Self::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
            frame: self.frame,
            surface: self.surface,
            binding: self.binding,
            mounted_instance: self.mounted_instance,
            node_receipt: self.node_receipt,
            bounds: geometry.bounds,
            clip_bounds: geometry.clip,
            order: self.order.presented_within_portal(),
        })
        .map(Some)
    }
}

impl UiMountedHitTestTable {
    pub const MAX_ROWS: usize = 65_536;

    pub fn empty() -> Self {
        Self {
            rows: std::sync::Arc::from([]),
        }
    }

    #[doc(hidden)]
    pub fn from_runtime_mounting(rows: Vec<UiMountedHitTestMechanic>) -> Option<Self> {
        (rows.len() <= Self::MAX_ROWS).then(|| Self { rows: rows.into() })
    }

    pub fn rows(&self) -> &[UiMountedHitTestMechanic] {
        &self.rows
    }

    pub fn resolve(
        &self,
        reference: UiMountedHitTestReference,
    ) -> Option<&UiMountedHitTestMechanic> {
        usize::try_from(reference.index())
            .ok()
            .and_then(|index| self.rows.get(index))
    }
}

fn semantic_digest(input: &UiMountedHitTestCompletionInput) -> u64 {
    [
        input.frame.diagnostic_value(),
        input.surface.diagnostic_value(),
        input.binding.diagnostic_value(),
        input.mounted_instance.diagnostic_value(),
        input.node_receipt.diagnostic_value(),
        u64::from(input.bounds.x().to_bits()),
        u64::from(input.bounds.y().to_bits()),
        u64::from(input.bounds.width().to_bits()),
        u64::from(input.bounds.height().to_bits()),
        u64::from(input.clip_bounds.x().to_bits()),
        u64::from(input.clip_bounds.y().to_bits()),
        u64::from(input.clip_bounds.width().to_bits()),
        u64::from(input.clip_bounds.height().to_bits()),
        input.order.presentation_layer_digest(),
        u64::from(input.order.rank()),
    ]
    .into_iter()
    .fold(0x6869_745f_7465_7374, |digest, value| {
        (digest ^ value).wrapping_mul(0x100000001b3)
    })
}
