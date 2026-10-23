//! Application-aftermath declaration surface.
//!
//! Domains declare correction authority and correction mechanism as separate
//! axes. Published law-14 posture names are derived only at installation.

mod contract;
mod correction_binding;
pub use correction_binding::{
    dispatch_application_correction, ApplicationCorrectionBinding,
    ApplicationCorrectionDerivationDenial, ApplicationCorrectionDispatch,
    ApplicationCorrectionPreImage, ApplicationCorrectionSourceMaterial,
    ApplicationCorrectionVisitor, ApplicationInverseBinding, DeclaredCorrectionBinding,
    DeclaredCorrectionBindingParts,
};
pub use correction_binding::{DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts};
mod correction_authority;
pub use correction_binding::{
    ApplicationCorrectionOutput, ApplicationCorrectionOutputField, DeclaredCorrectionOutputField,
    DeclaredCorrectionOutputFieldParts,
};
pub use correction_binding::{
    ApplicationCorrectionSourceFieldDenial, ApplicationCorrectionSourceOutputs,
};
mod correction_mechanism;
mod postcondition;
mod reconciliation;

#[cfg(test)]
mod recorded_inverse_tests;

pub use contract::{DeclaredApplicationAftermathContract, PortableApplicationAftermathContract};
pub use correction_authority::DeclaredCorrectionAuthority;
pub use correction_mechanism::{
    DeclaredCompensation, DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef,
    DeclaredPreImageDemand, DeclaredPreImageDemandDenial, DeclaredPreImageLocus,
    DeclaredRecordedInverse, PortableCorrectionMechanism, PortablePreImageDemand,
    PortablePreImageLocus, PortableRecordedInverse,
};
pub use postcondition::DeclaredAftermathPostcondition;
pub use reconciliation::DeclaredReconciliationProcedure;
