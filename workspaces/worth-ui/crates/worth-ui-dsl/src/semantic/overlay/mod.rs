mod backdrop;
mod extent;
mod motion;
mod placement;
mod presence;
mod scope;

pub use backdrop::{UiBackdropDeclaration, UiBackdropDeclarationDenial};
pub use extent::{UiBackdropExtentBasis, UiMosaicRegionDeclarationIdentity};
pub use motion::UiBackdropMotionBasis;
pub use placement::{
    UiBackdropPlacement, UiOverlayAnchor, UiOverlayPortalParticipant, UiOverlayRelation,
    UiOverlayRelationAdmissionDenial, UiOverlayRelationGraph, UiOverlayRelationKind,
};
pub use presence::UiBackdropPresenceBasis;
pub use scope::{
    UiBackdropIdentity, UiBackdropScope, UiPortalDeclarationId,
    UiSemanticSurfaceDeclarationIdentity,
};
