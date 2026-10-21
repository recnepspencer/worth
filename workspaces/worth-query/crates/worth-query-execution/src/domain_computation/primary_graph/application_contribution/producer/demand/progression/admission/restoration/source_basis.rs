//! Admit checkpoint source identity on the same basis rule as selection.

use worth_runtime_world::facade::ProductBranchObservation;

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::selection::source_basis_is_admitted;

use super::WorthQueryOutputDemandDenial;

pub(super) fn admit(
    source: &WorthQueryProductBranchReadIdentity,
    current: &ProductBranchObservation,
    owner_retained_program_basis: bool,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    // Inspect both identity/header paths before measuring variable branch text.

    // Compare borrowed fields, with no identity copies. Retained custody only
    // compares the branch owner/name and the incarnation; a direct read also
    // compares generation, commit and composite basis.

    Ok(source_basis_is_admitted(
        source,
        current,
        owner_retained_program_basis,
    ))
}
