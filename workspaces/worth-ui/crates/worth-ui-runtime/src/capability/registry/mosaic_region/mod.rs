mod descriptor;
mod frozen_mosaic_region_capabilities;
mod mosaic_region_registry;
mod registration;
mod seam_paint;
mod seam_registration;

pub use descriptor::{
    MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture,
    MosaicRegionKindDescriptor, MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership,
    MosaicSizingBehavior, UiScrollAxisSupport, UiScrollChromeContract,
    UiScrollChromeContractDenial, UiScrollLineExtent, UiScrollLineExtentDenial,
    UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS,
};
pub(crate) use frozen_mosaic_region_capabilities::UiAuthoredScrollRegionClauses;
pub use frozen_mosaic_region_capabilities::{
    FrozenMosaicRegionCapabilities, UiAuthoredScrollRegionCause, UiAuthoredScrollRegionDenial,
};
pub(crate) use mosaic_region_registry::MosaicRegionRegistry;
pub(crate) use registration::MosaicRegionAcceptedRegistrationProof;
pub use seam_paint::{
    MosaicExteriorCorner, MosaicExteriorCornerPosture, MosaicSeamPaintContract,
    MosaicSeamPaintContractDenial, MosaicSeamPaintOwner, MosaicSharedEdge,
};
pub(crate) use seam_registration::MosaicSeamPaintAcceptedRegistrationProof;
