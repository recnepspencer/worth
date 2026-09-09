use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceDamageAttribution, UiAppearanceVisualBounds, UiMountedAppearanceColor,
    UiMountedAppearanceFrame, UiMountedAppearanceMechanic, UiMountedAppearanceMechanicIdentity,
    UiMountedAppearanceOpacity, UiMountedBackdropAppearanceAttribution, UiMountedBackdropIdentity,
    UiMountedInstanceIdentity, UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedSurfacePaint, UiMountedTextPaintSpanIdentity,
    UiOverlayPlacementReceipt, UiSemanticSurfaceIdentity,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedAppearanceNodeInput {
    pub(super) geometry_input: Option<super::UiMountedAppearanceGeometryInput>,
    pub(super) issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) node_receipt: UiMountedNodeReceiptIdentity,
    pub(super) projection: UiMountedNodeAppearanceAttribution,
    pub(super) bounds: UiAppearanceAllocationBounds,
    pub(super) clip: super::UiMountedAppearanceClip,
    pub(super) surface_paint_order: Option<u32>,
    pub(super) radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    pub(super) surface_border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges,
    pub(super) surface_border_omissions:
        Box<[worth_ui_host_contract::UiMountedSurfaceBorderOmission]>,
    pub(super) surface_paint: Option<UiMountedSurfacePaint>,
    pub(super) outline: Option<UiMountedAppearanceOutlineInput>,
    pub(super) text_foregrounds: Box<[UiMountedAppearanceTextForegroundInput]>,
    pub(super) appearance_opacity: UiMountedAppearanceOpacity,
    pub(super) motion_opacity: Option<u16>,
    pub(super) semantic_digest: u64,
    pub(super) portal_instance: Option<UiMountedInstanceIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceOutlineInput {
    pub(super) geometry: worth_ui_host_contract::UiAppearanceOutlineGeometry,
    pub(super) color: UiMountedAppearanceColor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceTextForegroundInput {
    pub(super) span: UiMountedTextPaintSpanIdentity,
    pub(super) command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    pub(super) geometry: std::sync::Arc<[super::UiMountedAppearanceTextGeometry]>,
    pub(super) foreground: UiMountedAppearanceColor,
    pub(super) motion_opacity: Option<u16>,
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
    pub(super) motion_opacity: Option<u16>,
    pub(in crate::mounting::projection) motion_target: Option<UiMountedInstanceIdentity>,
    pub(super) attribution: UiMountedBackdropAppearanceAttribution,
    pub(super) semantic_digest: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceSurfaceOverlayInput {
    pub(in crate::mounting::projection) semantic_surface: UiSemanticSurfaceIdentity,
    pub(in crate::mounting::projection) portal_revision: u64,
    pub(in crate::mounting::projection) backdrop_revision: u64,
    pub(in crate::mounting::projection) portal_instances: Box<[UiMountedInstanceIdentity]>,
    pub(in crate::mounting::projection) backdrops: Box<[UiMountedAppearanceBackdropInput]>,
    pub(in crate::mounting::projection) bottom_to_top:
        Box<[worth_ui_host_contract::UiOverlayParticipantIdentity]>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum UiMountedAppearanceDamageShape {
    /// Exact target/span coverage is resolved at the text presentation boundary.
    TextForeground(std::sync::Arc<[super::UiMountedAppearanceTextGeometry]>),
    Visual {
        bounds: UiMountedAppearanceVisualBounds,
        attribution: UiAppearanceDamageAttribution,
    },
    Backdrop {
        extent: UiAppearanceBackdropExtent,
        clip: UiAppearanceClip,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiMountedAppearanceVisualBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl UiMountedAppearanceVisualBounds {
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
        clip: UiAppearanceClip,
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
            damage: UiMountedAppearanceDamageShape::Backdrop { extent, clip },
        }
    }

    pub(super) fn identity(&self) -> &UiMountedAppearanceMechanicIdentity {
        &self.identity
    }
    pub(super) const fn node_receipt(&self) -> Option<UiMountedNodeReceiptIdentity> {
        self.node_receipt
    }
    #[cfg(test)]
    pub(super) const fn projection(&self) -> Option<UiMountedNodeAppearanceAttribution> {
        self.projection
    }
    pub(super) fn mechanic(&self) -> &UiMountedAppearanceMechanic {
        &self.mechanic
    }
    pub(super) const fn damage(&self) -> &UiMountedAppearanceDamageShape {
        &self.damage
    }

    pub(super) fn same_physical_output(&self, other: &Self) -> bool {
        super::mechanic_equivalence::same_physical_output(&self.mechanic, &other.mechanic)
            && self.damage == other.damage
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedAppearanceFacts {
    geometry_inputs: Box<[super::UiMountedAppearanceGeometryInput]>,
    frame: UiMountedAppearanceFrame,
    records: Box<[UiMountedAppearanceFact]>,
}

impl UiMountedAppearanceFacts {
    pub(super) fn new(
        frame: UiMountedAppearanceFrame,
        records: Vec<UiMountedAppearanceFact>,
        geometry_inputs: Box<[super::UiMountedAppearanceGeometryInput]>,
    ) -> Self {
        Self {
            geometry_inputs,
            frame,
            records: records.into_boxed_slice(),
        }
    }

    pub(super) fn matches_geometry_input(
        &self,
        input: &super::UiMountedAppearanceGeometryInput,
    ) -> bool {
        self.geometry_inputs.contains(input)
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
