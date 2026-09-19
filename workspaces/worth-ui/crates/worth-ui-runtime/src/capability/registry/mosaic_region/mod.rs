mod descriptor;
mod frozen_mosaic_region_capabilities;
mod mosaic_region_registry;
mod registration;
mod seam_paint;
mod seam_registration;

pub use descriptor::{
    MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture,
    MosaicRegionKindDescriptor, MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership,
    MosaicSizingBehavior,
};
pub use frozen_mosaic_region_capabilities::FrozenMosaicRegionCapabilities;
pub(crate) use mosaic_region_registry::MosaicRegionRegistry;
pub(crate) use registration::MosaicRegionAcceptedRegistrationProof;
pub use seam_paint::{
    MosaicExteriorCorner, MosaicExteriorCornerPosture, MosaicSeamPaintContract,
    MosaicSeamPaintContractDenial, MosaicSeamPaintOwner, MosaicSharedEdge,
};
pub(crate) use seam_registration::MosaicSeamPaintAcceptedRegistrationProof;
