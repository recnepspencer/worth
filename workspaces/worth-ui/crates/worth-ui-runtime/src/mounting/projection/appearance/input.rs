#![allow(
    dead_code,
    reason = "Gate 1 retains owner-authored appearance input constructors for later mounting proofs"
)]

#[cfg(test)]
use super::fact::{
    UiMountedAppearanceBackdropInput, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput, UiMountedAppearanceOutlineInput, UiMountedAppearanceOverlayInput,
    UiMountedAppearancePointerInput, UiMountedAppearanceTextForegroundInput,
};
#[cfg(test)]
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiMountedAppearanceColor, UiMountedAppearanceOpacity, UiMountedBackdropAppearanceAttribution,
    UiMountedBackdropIdentity, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedLayerProjection, UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIdentity,
    UiMountedNodeReceiptIssuer, UiMountedPresentationAttemptIdentity, UiMountedSurfacePaint,
    UiMountedTextPaintSpanIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
    UiSemanticSurfaceIdentity,
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
