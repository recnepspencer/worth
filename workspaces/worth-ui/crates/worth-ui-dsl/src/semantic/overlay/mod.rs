mod authored;
mod backdrop;
mod extent;
mod motion;
mod placement;
mod presence;
mod relation_spec;
mod scope;

pub use authored::{
    UiBackdropDeclarationAuthoring, UiBackdropDeclarationAuthoringDenial,
    UiStaticBackdropDeclaration, UiStaticBackdropExtent, UiStaticBackdropMotion,
    UiStaticBackdropPlacement, UiStaticBackdropPresence, UiStaticBackdropScope,
};
pub use backdrop::{UiBackdropDeclaration, UiBackdropDeclarationDenial};
pub use extent::{UiBackdropExtentBasis, UiMosaicRegionDeclarationIdentity};
pub use motion::UiBackdropMotionBasis;
pub use placement::{
    UiBackdropPlacement, UiOverlayRelationAdmissionDenial, UiOverlayRelationGraph,
};
pub use presence::UiBackdropPresenceBasis;
pub use relation_spec::{
    UiStaticOverlayRelation, UiStaticOverlayRelationGraph, UiStaticOverlayRelationGraphDenial,
    UiStaticOverlayRelationKind,
};
pub use scope::{
    UiBackdropIdentity, UiBackdropScope, UiPortalDeclarationId,
    UiSemanticSurfaceDeclarationIdentity,
};
