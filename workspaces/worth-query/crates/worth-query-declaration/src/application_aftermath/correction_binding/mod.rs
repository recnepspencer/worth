//! Typed inverse meaning, visited by the audience that owns ordinary preparation.
mod disclosure;
mod dispatch;
mod inverse;
mod source_material;
mod source_output;
pub use source_material::{
    ApplicationCorrectionSourceFieldDenial, ApplicationCorrectionSourceMaterial,
    ApplicationCorrectionSourceOutputs,
};
pub use source_output::{
    ApplicationCorrectionOutput, ApplicationCorrectionOutputField, DeclaredCorrectionOutputField,
    DeclaredCorrectionOutputFieldParts,
};
mod visitor;

pub use disclosure::{DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts};
pub use dispatch::{dispatch_application_correction, ApplicationCorrectionDispatch};

pub use inverse::{
    ApplicationCorrectionBinding, ApplicationCorrectionDerivationDenial,
    ApplicationCorrectionPreImage, ApplicationInverseBinding, DeclaredCorrectionBinding,
    DeclaredCorrectionBindingParts,
};
pub use visitor::ApplicationCorrectionVisitor;
