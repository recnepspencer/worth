//! Admit checkpoint source identity on the same basis rule as selection.

use std::mem::size_of;

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::{
    CompositeBasisKey, CompositeCommitIdentity, ProductBranchIncarnation, ProductBranchObservation,
    ProductBranchReferenceGeneration, RuntimeWorldOwnerIdentity,
};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::demand::selection::source_basis_is_admitted,
    output_lineage::invalidation::InvalidationEditAdmission,
};

use super::{restoration_resource_denial, WorthQueryOutputDemandDenial};

pub(super) fn admit(
    source: &WorthQueryProductBranchReadIdentity,
    current: &ProductBranchObservation,
    owner_retained_program_basis: bool,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    // Inspect both identity/header paths before measuring variable branch text.
    admission
        .charge_external_work(8)
        .map_err(restoration_resource_denial)?;
    let source_width = source.branch_identity().name().as_str().len();
    let current_width = current.branch_identity().name().as_str().len();
    // Compare borrowed fields, with no identity copies. Retained custody only
    // compares the branch owner/name and the incarnation; a direct read also
    // compares generation, commit and composite basis.
    let occurrence_width =
        size_of::<RuntimeWorldOwnerIdentity>() + size_of::<ProductBranchIncarnation>();
    let fixed_width = if owner_retained_program_basis {
        occurrence_width
    } else {
        occurrence_width
            + size_of::<ProductBranchReferenceGeneration>()
            + size_of::<CompositeCommitIdentity>()
            + size_of::<CompositeBasisKey>()
    };
    let comparison_work = fixed_width
        .checked_mul(2)
        .and_then(|work| work.checked_add(source_width))
        .and_then(|work| work.checked_add(current_width))
        .and_then(|work| work.checked_add(8))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(|| restoration_resource_denial(CompanionPreflightStop::WorkCounterOverflow))?;
    admission
        .charge_external_work(comparison_work)
        .map_err(restoration_resource_denial)?;
    Ok(source_basis_is_admitted(
        source,
        current,
        owner_retained_program_basis,
    ))
}
