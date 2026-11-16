//! Application-aftermath declaration surface.
//!
//! Domains declare correction authority and correction mechanism as separate
//! axes. Published law-14 posture names are derived only at installation.

mod contract;
// Descriptive historical endpoints can be carried by an application using only
// the declaration and host facades; these identifiers grant no current handle.
pub use worth_foundational::facade::{EntityId, PartitionId};
mod relation_material;
pub use relation_material::ApplicationCorrectionRelationMaterial;
mod correction_binding;
pub use correction_binding::{
    dispatch_application_correction, ApplicationCorrectionBinding,
    ApplicationCorrectionDerivationDenial, ApplicationCorrectionDispatch,
    ApplicationCorrectionFieldPreImage, ApplicationCorrectionPreImage,
    ApplicationCorrectionSourceMaterial, ApplicationCorrectionVisitor, ApplicationInverseBinding,
    DeclaredCorrectionBinding, DeclaredCorrectionBindingParts,
};
pub use correction_binding::{
    dispatch_application_reapplication, ApplicationReapplicationDispatch,
    ApplicationReapplicationVisitor,
};
pub use correction_binding::{
    ApplicationAuthoredCorrection, ApplicationAuthoredCorrectionFamily,
    ApplicationAuthoredLogicalKey, DeclaredAuthoredCorrection, DeclaredAuthoredCorrectionParts,
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
    DeclaredCompensation, DeclaredCorrectionMechanism, DeclaredCorrectionMechanismDenial,
    DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand, DeclaredPreImageDemandDenial,
    DeclaredPreImageLocus, DeclaredPreImageRelationLocus, DeclaredRecordedInverse,
    PortableCorrectionMechanism, PortablePreImageDemand, PortablePreImageFieldOutput,
    PortablePreImageLocus, PortablePreImageRelationLocus, PortableRecordedInverse,
    PreImageOutputFieldCapture,
};
pub use postcondition::DeclaredAftermathPostcondition;
pub use reconciliation::DeclaredReconciliationProcedure;

pub(crate) use contract::AssociatedAftermathContract;
