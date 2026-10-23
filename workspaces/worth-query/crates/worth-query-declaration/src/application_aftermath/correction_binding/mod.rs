//! Typed inverse meaning, visited by the audience that owns ordinary preparation.
mod disclosure;
mod dispatch;
mod inverse;
mod source_material;
pub use source_material::ApplicationCorrectionSourceMaterial;
mod visitor;

pub use disclosure::{DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts};
pub use dispatch::{dispatch_application_correction, ApplicationCorrectionDispatch};

pub use inverse::{
    ApplicationCorrectionBinding, ApplicationCorrectionDerivationDenial,
    ApplicationCorrectionPreImage, ApplicationInverseBinding, DeclaredCorrectionBinding,
    DeclaredCorrectionBindingParts,
};
pub use visitor::ApplicationCorrectionVisitor;
