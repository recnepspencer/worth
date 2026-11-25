//! Attach a paid, empty Source image to the genuine live Native head that
//! carries none for the active registration: a recovered head, or one its
//! publishing registration left vacant.
use super::super::{
    cell::publication_admission::PreparedManagedSourceCell, CompanionBranchCell,
    CompanionPreflightStop as Stop, ManagedIndexSourceCellPlacement,
};
use super::{CompanionRegistrationState, CompanionRegistry, PublicationCompanionRegistrationStop};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::projection_view::{
    companion_slot::{CompanionBranchCellCore, CompanionRootImage},
    managed_source::ManagedSourceRootImage,
};
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::validation::candidate_source::image::{
    publication::ManagedIndexBranchImageOwnerPort as _, ManagedIndexBranchImage,
};
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::positioned_snapshot::PositionedRelationalSnapshot;
use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_contracts::internal_port::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;

/// Caller must hold owner + configuration epochs and declare `lookup_cleanup`
/// outside those guards so a failed participant placement drops after them.
#[allow(clippy::too_many_arguments)]
pub fn install_vacant_source_image(
    registry: &CompanionRegistry,
    runtime_instance_id: u64,
    execution: &ExecutionRequest<'_, '_>,
    root: &RelationalBranchRoot,
    selected: &PositionedRelationalSnapshot,
    matches_live_head: impl FnOnce(&BranchId, &RelationalBranchRoot) -> bool,
    lookup_cleanup: &mut Option<Box<dyn std::any::Any + Send + Sync>>,
) -> Result<ManagedSourceRootImage, Denial> {
    registry
        .with_head_exclusion(|state| {
            let CompanionRegistrationState::RequiredActive {
                generation,
                participant,
                client,
            } = state
            else {
                return Err(Denial::MissingParticipant);
            };
            let _client = client.upgrade().ok_or(Denial::Registration(
                PublicationCompanionRegistrationStop::RebindRequired,
            ))?;
            if selected.runtime_instance_id() != runtime_instance_id
                || !matches_live_head(selected.branch_id(), root)
            {
                return Err(Denial::ForeignSource);
            }
            if let Some(image) = root.managed_source_image.for_branch(selected.branch_id()) {
                image.admit_selected(selected, *generation)?;
                return Ok(image.into_owned());
            }
            let (cell, image) =
                prepare_cell(execution, selected, *generation).map_err(Denial::Publication)?;
            let placement = ManagedIndexSourceCellPlacement {
                cell: &cell,
                branch: selected.branch_id(),
                runtime: runtime_instance_id,
            };
            root.managed_source_image
                .attach_vacant(execution, selected.branch_id(), &image, || {
                    *lookup_cleanup =
                        Some(participant.prepare_managed_source_placement(execution, &placement)?);
                    Ok(())
                })
                .map_err(Denial::Publication)?;
            cell.installed();
            Ok(image)
        })
        .map_err(Denial::Registration)?
}

fn prepare_cell(
    execution: &ExecutionRequest<'_, '_>,
    selected: &PositionedRelationalSnapshot,
    generation: u64,
) -> Result<(PreparedManagedSourceCell, ManagedSourceRootImage), Stop> {
    let bytes = [
        size_of::<CompanionBranchCellCore>(),
        size_of::<CompanionRootImage>(),
        PreparedManagedSourceCell::metadata_bytes(),
        size_of::<worth_execution::ExecutionMemoryReservation>(),
    ]
    .into_iter()
    .try_fold(0usize, |total, bytes| {
        total.checked_add(bytes.checked_add(2 * size_of::<usize>())?)
    })
    .and_then(|bytes| bytes.checked_add(selected.branch_id().0.len().checked_mul(2)?))
    .and_then(|bytes| u64::try_from(bytes).ok())
    .ok_or(Stop::PreparationMemoryCounterOverflow)?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| Stop::Execution(cause.into()))?
        .map(Arc::new);
    let payload =
        ManagedIndexBranchImage::retain_header(execution, ManagedIndexBranchImage::default())?;
    let mut cell = CompanionBranchCell::new(selected, generation, payload);
    CompanionBranchCellCore::bind_unshared_placement_funding(&mut cell.core, memory);
    cell.core.retain_source_affinity(execution)?;
    let image = ManagedSourceRootImage::capture_current(&cell.core)
        .expect("fresh readmitted cell retained its original affinity");
    Ok((PreparedManagedSourceCell::new(cell), image))
}
