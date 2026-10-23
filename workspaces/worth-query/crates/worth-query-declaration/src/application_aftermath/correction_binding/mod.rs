//! Typed inverse meaning, visited by the audience that owns ordinary preparation.
mod authored_identity;
mod disclosure;
pub use authored_identity::{
    ApplicationAuthoredCorrection, ApplicationAuthoredCorrectionFamily,
    ApplicationAuthoredLogicalKey, DeclaredAuthoredCorrection, DeclaredAuthoredCorrectionParts,
};
mod dispatch;
mod inverse;
mod reapplication;
pub use reapplication::{
    dispatch_application_reapplication, ApplicationReapplicationDispatch,
    ApplicationReapplicationVisitor,
};
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
