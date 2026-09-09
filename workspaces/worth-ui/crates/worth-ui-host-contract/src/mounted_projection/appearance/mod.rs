mod backdrop;
mod bounds;
mod change;
mod color;
mod compositing;
mod frame;
mod geometry_qualification;
mod logical_length;
mod native_profile;
mod node_attribution;
mod opacity;
mod outline;
mod outline_geometry;
mod overlay_order;
mod pointer_affordance;
mod portal_surface;
mod radii;
mod surface;
mod surface_border;
mod text_damage;
mod text_foreground;
mod unpublished;

pub use backdrop::{
    UiMountedBackdropAppearanceAttribution, UiMountedBackdropCompletionDenial,
    UiMountedBackdropCompletionInput, UiMountedBackdropIdentity, UiMountedBackdropMechanic,
    UiMountedBackdropScope, UiOverlayPlacementReceipt,
};
pub use bounds::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceDamageAttribution, UiAppearanceDamageRegion, UiAppearanceEmptyRegion,
    UiAppearanceGeometryOverflow, UiAppearanceVisualBounds,
};
pub use change::{
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceMechanicIdentity,
};
pub use color::UiMountedAppearanceColor;
pub use compositing::{
    compose_source_over, SRGB_GAMMA_DENOMINATOR, SRGB_GAMMA_NUMERATOR,
    SRGB_LINEAR_SCALE_DENOMINATOR, SRGB_LINEAR_SCALE_NUMERATOR, SRGB_LINEAR_THRESHOLD_DENOMINATOR,
    SRGB_LINEAR_THRESHOLD_NUMERATOR, SRGB_OFFSET_DENOMINATOR, SRGB_OFFSET_NUMERATOR,
    SRGB_SCALE_DENOMINATOR, SRGB_SCALE_NUMERATOR,
};
pub use frame::{
    UiMountedAppearanceFrame, UiMountedAppearanceFrameDenial,
    UiMountedAppearancePredecessorManifest, UiMountedAppearanceWork,
    UiMountedAppearanceWorkPosture,
};
pub use geometry_qualification::{
    UiHostAppearanceGeometryQualification, UiHostAppearanceGeometryQualificationBasis,
    UiHostAppearanceGeometryQualificationDenial, UiHostAppearanceScaleDenial,
    UiHostAppearanceScaleGeometryQualification, UI_HOST_APPEARANCE_GEOMETRY_ROW_CAPACITY,
};
pub use logical_length::{
    UiAppearanceLogicalLength, UiAppearanceNegativeLength,
    UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};
pub use native_profile::{
    UiHostAppearanceMechanicFamily, UiHostAppearanceProfileContract, UiHostAppearanceProfileDenial,
    UiHostAppearanceProfilePosture,
};
pub use node_attribution::UiMountedNodeAppearanceAttribution;
pub use opacity::UiMountedAppearanceOpacity;
pub use outline::{
    UiMountedOutlineAppearanceCompletionDenial, UiMountedOutlineAppearanceCompletionInput,
    UiMountedOutlineAppearanceMechanic,
};
pub use outline_geometry::{UiAppearanceOutlineGeometry, UiAppearanceOutlineGeometryDenial};
pub use overlay_order::{
    UiMountedOverlayOrderMechanic, UiMountedOverlayOrderMechanicDenial,
    UiOverlayParticipantIdentity,
};
pub use pointer_affordance::{
    UiHostPrimaryPointerKind, UiMountedPointerAffordanceMechanic, UiPointerAffordanceFamily,
};
pub use portal_surface::{
    UiMountedPortalSurfaceAppearanceCompletionDenial, UiMountedPortalSurfaceAppearanceMechanic,
};
pub use radii::UiAppearanceNormalizedLogicalRadii;
pub use surface::{
    UiMountedSurfaceAppearanceCompletionDenial, UiMountedSurfaceAppearanceCompletionInput,
    UiMountedSurfaceAppearanceMechanic, UiMountedSurfaceBorderEdges, UiMountedSurfacePaint,
};
pub use surface_border::{UiMountedSurfaceBorderOmission, UiMountedSurfaceBorderSide};
pub use text_damage::{UiAppearanceTextDamageRequirement, UiAppearanceTextDamageTransition};
pub use text_foreground::{
    UiMountedTextForegroundAppearanceCompletionDenial,
    UiMountedTextForegroundAppearanceCompletionInput, UiMountedTextForegroundAppearanceMechanic,
};
pub use unpublished::{
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFragmentIdentity,
    UiUnpublishedAppearanceFrameProjection, UiUnpublishedAppearanceFrameProjectionDenial,
    UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY,
};
