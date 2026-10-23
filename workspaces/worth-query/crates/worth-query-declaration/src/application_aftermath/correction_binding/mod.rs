//! Typed inverse meaning, visited by the audience that owns ordinary preparation.
mod disclosure;
mod dispatch;
mod inverse;
mod visitor;

pub use disclosure::{DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts};
pub use dispatch::{dispatch_application_correction, ApplicationCorrectionDispatch};

pub use inverse::{
    ApplicationCorrectionBinding, ApplicationCorrectionDerivationDenial,
    ApplicationCorrectionPreImage, ApplicationInverseBinding, DeclaredInverseBinding,
    DeclaredInverseBindingParts,
};
pub use visitor::ApplicationCorrectionVisitor;
