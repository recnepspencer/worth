//! Moved whole from `domain_computation::primary_graph::application_installation::program::program_owner`: the items below and every impl they own.

#[derive(Debug)]
pub enum WorthQuerySelectedProgramOwnerDenial {
    ProductSelection(
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    ),
    Inspection(crate::domain_computation::primary_graph::vocabulary::product_activation::inspection::WorthQuerySelectedProgramInspectionDenial),
    InstalledOwnerUnavailable,
    UndeclaredOutputRoot,
}
