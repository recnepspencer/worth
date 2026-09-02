mod backdrop;
mod extent;
mod motion;
mod placement;
mod presence;
mod scope;

pub use backdrop::{UiBackdropDeclaration, UiBackdropDeclarationDenial, UiBackdropIdentity};
pub use extent::{
    UiBackdropExtentBasis, UiMosaicRegionDeclarationIdentity, UiSemanticSurfaceDeclarationIdentity,
};
pub use motion::UiBackdropMotionBasis;
pub use placement::{
    UiBackdropPlacement, UiOverlayAnchor, UiOverlayPortalParticipant, UiOverlayRelation,
    UiOverlayRelationAdmissionDenial, UiOverlayRelationGraph, UiOverlayRelationKind,
};
pub use presence::{UiBackdropPresenceBasis, UiPortalDeclarationId};
pub use scope::UiBackdropScope;
