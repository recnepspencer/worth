mod appearance;
mod geometry;
mod headless_cache;
mod headless_observation;
mod hit_test;
mod identity_overlay;
mod participation;
#[cfg(test)]
mod portal_child_presentation_tests;
mod portal_overlay;
mod portal_presentation;
mod preview;
mod resource;
mod semantic_text;
mod static_paint;
mod tables;
mod view;

pub use appearance::{
    compose_source_over, UiAppearanceAllocationBounds, UiAppearanceBackdropExtent,
    UiAppearanceClip, UiAppearanceDamageAttribution, UiAppearanceDamageRegion,
    UiAppearanceEmptyRegion, UiAppearanceGeometryOverflow, UiAppearanceLogicalLength,
    UiAppearanceNegativeLength, UiAppearanceNormalizedLogicalRadii, UiAppearanceOutlineGeometry,
    UiAppearanceOutlineGeometryDenial, UiAppearanceVisualBounds, UiHostAppearanceMechanicFamily,
    UiHostAppearanceProfileContract, UiHostAppearanceProfileDenial, UiHostAppearanceProfilePosture,
    UiHostPrimaryPointerKind, UiMountedAppearanceColor, UiMountedAppearanceFrame,
    UiMountedAppearanceFrameDenial, UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceMechanicIdentity, UiMountedAppearanceOpacity,
    UiMountedAppearancePredecessorManifest, UiMountedAppearanceWork,
    UiMountedAppearanceWorkPosture, UiMountedBackdropAppearanceAttribution,
    UiMountedBackdropCompletionDenial, UiMountedBackdropCompletionInput, UiMountedBackdropIdentity,
    UiMountedBackdropMechanic, UiMountedBackdropScope, UiMountedNodeAppearanceAttribution,
    UiMountedOutlineAppearanceCompletionDenial, UiMountedOutlineAppearanceCompletionInput,
    UiMountedOutlineAppearanceMechanic, UiMountedOverlayOrderMechanic,
    UiMountedOverlayOrderMechanicDenial, UiMountedPointerAffordanceMechanic,
    UiMountedPortalSurfaceAppearanceCompletionDenial, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceCompletionDenial, UiMountedSurfaceAppearanceCompletionInput,
    UiMountedSurfaceAppearanceMechanic, UiMountedSurfacePaint,
    UiMountedTextForegroundAppearanceCompletionDenial,
    UiMountedTextForegroundAppearanceCompletionInput, UiMountedTextForegroundAppearanceMechanic,
    UiOverlayParticipantIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
    SRGB_GAMMA_DENOMINATOR, SRGB_GAMMA_NUMERATOR, SRGB_LINEAR_SCALE_DENOMINATOR,
    SRGB_LINEAR_SCALE_NUMERATOR, SRGB_LINEAR_THRESHOLD_DENOMINATOR,
    SRGB_LINEAR_THRESHOLD_NUMERATOR, SRGB_OFFSET_DENOMINATOR, SRGB_OFFSET_NUMERATOR,
    SRGB_SCALE_DENOMINATOR, SRGB_SCALE_NUMERATOR, UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};
pub use geometry::{
    UiMountedAllocationBasis, UiMountedAllocationProjection, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedGeometryDenial,
    UiMountedGeometryPosture, UiMountedTransformProjection,
};
pub use headless_cache::{
    UiHeadlessMountedResourceHandle, WorthUiHeadlessMountedResourceCache,
    WorthUiMountedResourceCacheDenial,
};
pub use headless_observation::{
    UiHeadlessMountedParticipationRecord, WorthUiHeadlessMountedProjectionRecord,
};
pub use hit_test::{
    UiMountedHitTestCompletionDenial, UiMountedHitTestCompletionInput, UiMountedHitTestMechanic,
    UiMountedHitTestOrder, UiMountedHitTestProjection, UiMountedHitTestReference,
    UiMountedHitTestTable,
};
pub use identity_overlay::{
    UiMountedClientCoordinateBasis, UiMountedClientPhysicalRect, UiMountedIdentityOverlayMechanic,
    UiMountedIdentityOverlayMechanicInput,
};
pub use participation::{
    UiMountedAccessibilityProjection, UiMountedDiagnosticProjection, UiMountedDiagnosticReference,
    UiMountedMechanicalRole, UiMountedMotionProjection, UiMountedOmissionReason,
    UiMountedPaintProjection, UiMountedParticipation, UiMountedParticipationFact,
    UiMountedParticipationInput, UiMountedParticipationStatus, UiMountedProjectionAudience,
};
pub use portal_overlay::{
    UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionDenial,
    UiMountedPortalOverlayCompletionInput, UiMountedPortalOverlayLifecyclePosture,
    UiMountedPortalOverlayMechanic, UiMountedPortalOverlayReference,
    UiMountedPortalOverlaySchemaVersion, UiMountedPortalOverlayTable,
};
pub use portal_presentation::UiMountedPortalPresentationAffinity;
pub use preview::UiMountedPreviewProjection;
pub use resource::{
    UiMountedResourceEntry, UiMountedResourceKind, UiMountedResourceReference,
    UiMountedResourceTable,
};
pub use semantic_text::{
    UiMountedCollectionRowCorrelation, UiMountedQualifiedTextResolver,
    UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiMountedSemanticTextReference, UiMountedSemanticTextTable,
    UiMountedSemanticTextTableDenial, UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity,
    UiMountedTextSchemaVersion, UiSemanticTextBaselinePosture, UiSemanticTextProfile,
    UiSemanticTextSlot, UiSemanticTextWrapPosture,
};
pub use static_paint::{
    UiMountedFilledRectCompletionDenial, UiMountedFilledRectCompletionInput,
    UiMountedFilledRectMechanic, UiMountedFilledRectReference, UiMountedFilledRectTable,
    UiMountedFilledRectTableDenial, UiMountedRgba8, UiMountedStaticPaintSchemaVersion,
};
pub use tables::{
    UiMountedClipProjection, UiMountedClipReference, UiMountedClipRow, UiMountedClipTable,
    UiMountedLayerProjection, UiMountedLayerReference, UiMountedLayerRow, UiMountedLayerTable,
    UiMountedPaintBatchReference, UiMountedPaintBatchRow, UiMountedPaintBatchTable,
    UiMountedPaintPrimitiveKind, UiMountedRealtimeBatchReference, UiMountedRealtimeBatchRow,
    UiMountedRealtimeBatchTable, UiMountedSpatialBatchReference, UiMountedSpatialBatchRow,
    UiMountedSpatialBatchTable, UiMountedTableProjectionStatus,
};
pub use view::{
    UiMountedDrawableReference, UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput,
    UiMountedProjectionView, UiMountedProjectionViewInput,
};
