use super::fact::UiMountedAppearanceNodeInput;
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiMountedNodeReceiptIdentity, UiMountedNodeReceiptIssuer,
    UiSemanticSurfaceIdentity,
};

#[cfg(test)]
use super::fact::{
    UiMountedAppearanceBackdropInput, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceOutlineInput, UiMountedAppearanceOverlayInput,
    UiMountedAppearancePointerInput, UiMountedAppearanceTextForegroundInput,
};
#[cfg(test)]
use worth_ui_host_contract::{
    UiAppearanceBackdropExtent, UiAppearanceClip, UiMountedAppearanceColor,
    UiMountedAppearanceOpacity, UiMountedBackdropAppearanceAttribution, UiMountedBackdropIdentity,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedLayerProjection,
    UiMountedNodeAppearanceAttribution, UiMountedPresentationAttemptIdentity,
    UiMountedSurfacePaint, UiMountedTextPaintSpanIdentity, UiOverlayPlacementReceipt,
    UiPointerAffordanceFamily,
};

#[cfg(test)]
impl UiMountedAppearanceLoweringInput {
    pub(crate) fn new(
        frame: UiMountedFrameIdentity,
        semantic_surface: UiSemanticSurfaceIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        nodes: Vec<UiMountedAppearanceNodeInput>,
        backdrops: Vec<UiMountedAppearanceBackdropInput>,
        overlay: UiMountedAppearanceOverlayInput,
    ) -> Self {
        Self {
            frame,
            semantic_surface,
            presentation,
            nodes,
            backdrops,
            overlay,
        }
    }
}

#[cfg(test)]
impl UiMountedAppearanceNodeInput {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        issuer: UiMountedNodeReceiptIssuer,
        semantic_surface: UiSemanticSurfaceIdentity,
        node_receipt: UiMountedNodeReceiptIdentity,
        projection: UiMountedNodeAppearanceAttribution,
        bounds: UiAppearanceAllocationBounds,
        clip: UiAppearanceClip,
        layer: UiMountedLayerProjection,
        radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
        surface_paint: Option<UiMountedSurfacePaint>,
        outline: Option<UiMountedAppearanceOutlineInput>,
        text_foregrounds: impl Into<Box<[UiMountedAppearanceTextForegroundInput]>>,
        pointer: Option<UiMountedAppearancePointerInput>,
        appearance_opacity: UiMountedAppearanceOpacity,
        motion_opacity: Option<UiMountedAppearanceOpacity>,
        semantic_digest: u64,
        portal_instance: Option<UiMountedInstanceIdentity>,
    ) -> Self {
        Self {
            issuer,
            semantic_surface,
            node_receipt,
            projection,
            bounds,
            clip,
            layer,
            radii,
            surface_paint,
            outline,
            text_foregrounds: text_foregrounds.into(),
            pointer,
            appearance_opacity,
            motion_opacity,
            semantic_digest,
            portal_instance,
        }
    }
}

impl UiMountedAppearanceNodeInput {
    pub(in crate::mounting::projection) fn from_runtime_mounting(
        issuer: UiMountedNodeReceiptIssuer,
        semantic_surface: UiSemanticSurfaceIdentity,
        node_receipt: UiMountedNodeReceiptIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        plan_digest: u64,
        allocation: worth_ui_host_contract::UiMountedAllocationProjection,
        static_paint: Option<super::super::static_paint::UiMountedStaticPaintSeed>,
    ) -> Result<Self, super::UiMountedAppearanceLoweringDenial> {
        let bounds = match allocation {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. }
            | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                bounds,
                ..
            } => bounds,
            worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                return Err(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
            }
        };
        let bounds = runtime_bounds(bounds)?;
        let projection =
            worth_ui_host_contract::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                issuer,
                graph_node.digest(),
                plan_digest,
            )
            .ok_or(super::UiMountedAppearanceLoweringDenial::NodeProjectionUnavailable)?;
        Ok(Self {
            issuer,
            semantic_surface,
            node_receipt,
            projection,
            bounds,
            clip: worth_ui_host_contract::UiAppearanceClip::new(
                bounds.x(),
                bounds.y(),
                bounds.width(),
                bounds.height(),
            )
            .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)?,
            layer: worth_ui_host_contract::UiMountedLayerProjection::Omitted(
                worth_ui_host_contract::UiMountedOmissionReason::NotDefinedByCurrentRuntime,
            ),
            radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
            ),
            surface_paint: static_paint.map(|seed| {
                worth_ui_host_contract::UiMountedSurfacePaint::Fill(
                    worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                        seed.color().channels(),
                    ),
                )
            }),
            outline: None,
            text_foregrounds: Box::new([]),
            pointer: None,
            appearance_opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
            motion_opacity: None,
            semantic_digest: plan_digest,
            portal_instance: None,
        })
    }
}

fn runtime_bounds(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Result<UiAppearanceAllocationBounds, super::UiMountedAppearanceLoweringDenial> {
    if bounds.posture() != worth_ui_host_contract::UiMountedGeometryPosture::Area {
        return Err(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable);
    }
    let x = exact_i32(bounds.x())?;
    let y = exact_i32(bounds.y())?;
    let width = exact_u32(bounds.width())?;
    let height = exact_u32(bounds.height())?;
    UiAppearanceAllocationBounds::new(x, y, width, height)
        .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}

fn exact_i32(value: f32) -> Result<i32, super::UiMountedAppearanceLoweringDenial> {
    let candidate = value as i32;
    (candidate as f32 == value)
        .then_some(candidate)
        .ok_or(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}

fn exact_u32(value: f32) -> Result<u32, super::UiMountedAppearanceLoweringDenial> {
    let candidate = value as u32;
    (candidate as f32 == value)
        .then_some(candidate)
        .ok_or(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}

#[cfg(test)]
impl UiMountedAppearanceBackdropInput {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        identity: UiMountedBackdropIdentity,
        semantic_surface: UiSemanticSurfaceIdentity,
        placement: UiOverlayPlacementReceipt,
        extent: UiAppearanceBackdropExtent,
        clip: UiAppearanceClip,
        background: UiMountedAppearanceColor,
        appearance_opacity: UiMountedAppearanceOpacity,
        motion_opacity: Option<UiMountedAppearanceOpacity>,
        attribution: UiMountedBackdropAppearanceAttribution,
        semantic_digest: u64,
    ) -> Self {
        Self {
            identity,
            semantic_surface,
            placement,
            extent,
            clip,
            background,
            appearance_opacity,
            motion_opacity,
            attribution,
            semantic_digest,
        }
    }
}

#[cfg(test)]
impl UiMountedAppearanceOverlayInput {
    pub(crate) fn new(
        semantic_surface: UiSemanticSurfaceIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        portal_revision: u64,
        backdrop_revision: u64,
        bottom_to_top: impl Into<Box<[worth_ui_host_contract::UiOverlayParticipantIdentity]>>,
    ) -> Self {
        Self {
            semantic_surface,
            presentation,
            portal_revision,
            backdrop_revision,
            bottom_to_top: bottom_to_top.into(),
        }
    }
}

#[cfg(test)]
impl UiMountedAppearanceOutlineInput {
    pub(crate) const fn new(
        geometry: worth_ui_host_contract::UiAppearanceOutlineGeometry,
        color: UiMountedAppearanceColor,
    ) -> Self {
        Self { geometry, color }
    }
}

#[cfg(test)]
impl UiMountedAppearanceTextForegroundInput {
    pub(crate) const fn new(
        span: UiMountedTextPaintSpanIdentity,
        foreground: UiMountedAppearanceColor,
    ) -> Self {
        Self { span, foreground }
    }
}

#[cfg(test)]
impl UiMountedAppearancePointerInput {
    pub(crate) const fn new(
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        surface: UiSemanticSurfaceIdentity,
        target: UiMountedInstanceIdentity,
        family: UiPointerAffordanceFamily,
    ) -> Self {
        Self {
            pointer,
            surface,
            target,
            family,
        }
    }
}
