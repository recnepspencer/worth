use super::basis_axis_validation::require_current_descriptor_axes;
use super::basis_identity_validation::require_local_branch_identity;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::branch::lifecycle::RelationalBranchLifecyclePosture;
use crate::branch::root::RelationalBranchRoot;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::schema::data::RelationalSchemaRegistry;

/// Revalidate an admitted basis against this owner's live branch cell.
///
/// A retained historical basis remains valid for exact retained reads; it
/// is not required to be the ambient current head. This narrower port is
/// for Runtime World admission of a current component tuple and therefore
/// checks the real owner cell before the tuple can be composed.
pub(crate) fn compare_current_exact(
    history: &HistorySubsystem,
    registry: &RelationalSchemaRegistry,
    runtime_id: u64,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<(), RelationalBranchBasisDenial> {
    let descriptor = basis.descriptor();
    if descriptor.runtime_instance_id() != runtime_id {
        return Err(RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: runtime_id,
            actual_runtime_instance_id: descriptor.runtime_instance_id(),
        });
    }
    require_local_branch_identity(runtime_id, history, basis.identity())?;
    let branch_cell = history
        .branch_cell(basis.identity().branch_id())
        .ok_or_else(|| {
            RelationalBranchBasisDenial::UnknownBranch(basis.identity().branch_id().clone())
        })?;
    match branch_cell.lifecycle_posture() {
        RelationalBranchLifecyclePosture::Live => {}
        RelationalBranchLifecyclePosture::Archived => {
            return Err(RelationalBranchBasisDenial::ArchivedBranch(
                basis.identity().branch_id().clone(),
            ));
        }
        RelationalBranchLifecyclePosture::Deleting => {
            return Err(RelationalBranchBasisDenial::DeletingBranch(
                basis.identity().branch_id().clone(),
            ));
        }
    }

    let snapshot = branch_cell.atomic_snapshot();
    let current_reference = snapshot.observation();
    let current_truth_version = snapshot.truth_version();
    let root = snapshot.root().unwrap_or_else(|| {
        RelationalBranchRoot::empty_with_schema(
            registry,
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
        )
    });
    require_current_descriptor_axes(descriptor, &current_reference, current_truth_version, &root)?;
    if !basis.is_current() {
        return Err(RelationalBranchBasisDenial::MixedAxis(
            crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::TruthVersion,
        ));
    }
    Ok(())
}
