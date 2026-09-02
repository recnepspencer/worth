use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceDamageAttribution, UiAppearanceVisualBounds, UiMountedAppearanceColor,
    UiMountedAppearanceFrame, UiMountedAppearanceMechanic, UiMountedAppearanceMechanicIdentity,
    UiMountedAppearanceOpacity, UiMountedBackdropAppearanceAttribution, UiMountedBackdropIdentity,
    UiMountedInstanceIdentity, UiMountedLayerProjection, UiMountedNodeAppearanceAttribution,
    UiMountedNodeReceiptIdentity, UiMountedPresentationAttemptIdentity, UiMountedSurfacePaint,
    UiMountedTextPaintSpanIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
    UiSemanticSurfaceIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceNodeInput {
    pub(super) issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) node_receipt: UiMountedNodeReceiptIdentity,
    pub(super) projection: UiMountedNodeAppearanceAttribution,
    pub(super) bounds: UiAppearanceAllocationBounds,
    pub(super) clip: UiAppearanceClip,
    pub(super) layer: UiMountedLayerProjection,
    pub(super) radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    pub(super) surface_paint: Option<UiMountedSurfacePaint>,
    pub(super) outline: Option<UiMountedAppearanceOutlineInput>,
    pub(super) text_foregrounds: Box<[UiMountedAppearanceTextForegroundInput]>,
    pub(super) pointer: Option<UiMountedAppearancePointerInput>,
    pub(super) appearance_opacity: UiMountedAppearanceOpacity,
    pub(super) motion_opacity: Option<UiMountedAppearanceOpacity>,
    pub(super) semantic_digest: u64,
    pub(super) portal_instance: Option<UiMountedInstanceIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceOutlineInput {
    pub(super) geometry: worth_ui_host_contract::UiAppearanceOutlineGeometry,
    pub(super) color: UiMountedAppearanceColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceTextForegroundInput {
    pub(super) span: UiMountedTextPaintSpanIdentity,
    pub(super) foreground: UiMountedAppearanceColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearancePointerInput {
    pub(super) pointer: worth_ui_host_contract::UiHostPointerIdentity,
    pub(super) surface: UiSemanticSurfaceIdentity,
    pub(super) target: UiMountedInstanceIdentity,
    pub(super) family: UiPointerAffordanceFamily,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceBackdropInput {
    pub(super) identity: UiMountedBackdropIdentity,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) placement: UiOverlayPlacementReceipt,
    pub(super) extent: UiAppearanceBackdropExtent,
    pub(super) clip: UiAppearanceClip,
    pub(super) background: UiMountedAppearanceColor,
    pub(super) appearance_opacity: UiMountedAppearanceOpacity,
    pub(super) motion_opacity: Option<UiMountedAppearanceOpacity>,
    pub(super) attribution: UiMountedBackdropAppearanceAttribution,
    pub(super) semantic_digest: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceOverlayInput {
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) presentation: UiMountedPresentationAttemptIdentity,
    pub(super) portal_revision: u64,
    pub(super) backdrop_revision: u64,
    pub(super) bottom_to_top: Box<[worth_ui_host_contract::UiOverlayParticipantIdentity]>,
}

#[derive(Debug)]
pub(crate) struct UiMountedAppearanceLoweringInput {
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) presentation: UiMountedPresentationAttemptIdentity,
    pub(super) nodes: Vec<UiMountedAppearanceNodeInput>,
    pub(super) backdrops: Vec<UiMountedAppearanceBackdropInput>,
    pub(super) overlay: UiMountedAppearanceOverlayInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiMountedAppearanceDamageShape {
    Visual {
        bounds: UiMountedAppearanceVisualBounds,
        attribution: UiAppearanceDamageAttribution,
    },
    Backdrop {
        extent: UiAppearanceBackdropExtent,
    },
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiMountedAppearanceVisualBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl UiMountedAppearanceVisualBounds {
    pub(super) const fn from_allocation(bounds: UiAppearanceAllocationBounds) -> Self {
        Self {
            x: bounds.x(),
            y: bounds.y(),
            width: bounds.width(),
            height: bounds.height(),
        }
    }

    pub(super) const fn from_visual(bounds: UiAppearanceVisualBounds) -> Self {
        Self {
            x: bounds.x(),
            y: bounds.y(),
            width: bounds.width(),
            height: bounds.height(),
        }
    }

    pub(super) const fn x(self) -> i32 {
        self.x
    }
    pub(super) const fn y(self) -> i32 {
        self.y
    }
    pub(super) const fn width(self) -> u32 {
        self.width
    }
    pub(super) const fn height(self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiMountedAppearanceFact {
    identity: UiMountedAppearanceMechanicIdentity,
    semantic_surface: UiSemanticSurfaceIdentity,
    node_receipt: Option<UiMountedNodeReceiptIdentity>,
    projection: Option<UiMountedNodeAppearanceAttribution>,
    backdrop_attribution: Option<UiMountedBackdropAppearanceAttribution>,
    semantic_digest: u64,
    mechanic: UiMountedAppearanceMechanic,
    damage: UiMountedAppearanceDamageShape,
}

impl UiMountedAppearanceFact {
    pub(super) fn node(
        semantic_surface: UiSemanticSurfaceIdentity,
        node_receipt: UiMountedNodeReceiptIdentity,
        projection: UiMountedNodeAppearanceAttribution,
        semantic_digest: u64,
        mechanic: UiMountedAppearanceMechanic,
        damage: UiMountedAppearanceDamageShape,
    ) -> Self {
        let identity = mechanic.identity();
        Self {
            identity,
            semantic_surface,
            node_receipt: Some(node_receipt),
            projection: Some(projection),
            backdrop_attribution: None,
            semantic_digest,
            mechanic,
            damage,
        }
    }

    pub(super) fn backdrop(
        semantic_surface: UiSemanticSurfaceIdentity,
        attribution: UiMountedBackdropAppearanceAttribution,
        semantic_digest: u64,
        mechanic: UiMountedAppearanceMechanic,
        extent: UiAppearanceBackdropExtent,
    ) -> Self {
        let identity = mechanic.identity();
        Self {
            identity,
            semantic_surface,
            node_receipt: None,
            projection: None,
            backdrop_attribution: Some(attribution),
            semantic_digest,
            mechanic,
            damage: UiMountedAppearanceDamageShape::Backdrop { extent },
        }
    }

    pub(super) fn identity(&self) -> &UiMountedAppearanceMechanicIdentity {
        &self.identity
    }
    pub(super) const fn semantic_surface(&self) -> UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub(super) const fn node_receipt(&self) -> Option<UiMountedNodeReceiptIdentity> {
        self.node_receipt
    }
    pub(super) const fn projection(&self) -> Option<UiMountedNodeAppearanceAttribution> {
        self.projection
    }
    pub(super) const fn backdrop_attribution(
        &self,
    ) -> Option<UiMountedBackdropAppearanceAttribution> {
        self.backdrop_attribution
    }
    pub(super) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
    pub(super) fn mechanic(&self) -> &UiMountedAppearanceMechanic {
        &self.mechanic
    }
    pub(super) const fn damage(&self) -> UiMountedAppearanceDamageShape {
        self.damage
    }

    pub(super) fn same_semantic_meaning(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.semantic_surface == other.semantic_surface
            && self.node_receipt.map(|receipt| receipt.mounted_instance())
                == other.node_receipt.map(|receipt| receipt.mounted_instance())
            && self
                .projection
                .map(|projection| (projection.identity(), projection.revision()))
                == other
                    .projection
                    .map(|projection| (projection.identity(), projection.revision()))
            && self.backdrop_attribution.map(|attribution| {
                (
                    attribution.semantic_surface(),
                    attribution.overlay_revision(),
                    attribution.identity(),
                    attribution.revision(),
                )
            }) == other.backdrop_attribution.map(|attribution| {
                (
                    attribution.semantic_surface(),
                    attribution.overlay_revision(),
                    attribution.identity(),
                    attribution.revision(),
                )
            })
            && self.semantic_digest == other.semantic_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceFacts {
    frame: UiMountedAppearanceFrame,
    records: Box<[UiMountedAppearanceFact]>,
}

impl UiMountedAppearanceFacts {
    pub(super) fn new(
        frame: UiMountedAppearanceFrame,
        records: Vec<UiMountedAppearanceFact>,
    ) -> Self {
        Self {
            frame,
            records: records.into_boxed_slice(),
        }
    }

    pub(super) const fn frame(&self) -> &UiMountedAppearanceFrame {
        &self.frame
    }
    pub(super) fn records(&self) -> &[UiMountedAppearanceFact] {
        &self.records
    }
    pub(super) fn record(
        &self,
        identity: &UiMountedAppearanceMechanicIdentity,
    ) -> Option<&UiMountedAppearanceFact> {
        self.records
            .iter()
            .find(|record| record.identity() == identity)
    }
}
