//! Attach a paid, empty Source image to the genuine recovered Native head.
use super::super::{
    cell::{
        publication_admission::PreparedManagedSourceCell, CompanionBranchCellCore,
        CompanionRootImage,
    },
    CompanionBranchCell, CompanionPreflightStop as Stop, ManagedIndexSourceCellPlacement,
    ManagedSourceRootImage,
};
use super::{CompanionRegistrationState, PublicationCompanionRegistrationStop};
use crate::{
    branch::RelationalBranchRoot,
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    validation::candidate_source::{
        ManagedIndexBranchImage, ManagedIndexSourcePreparationDenial as Denial,
    },
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) fn install_readmitted_source_image(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    root: &Arc<RelationalBranchRoot>,
    selected: &PositionedRelationalSnapshot,
) -> Result<ManagedSourceRootImage, Denial> {
    // A failed participant placement drops its lookup after Native's epochs.
    let mut lookup = None;
    let owner = runtime.owner_binding();
    let _operation = owner.admit().ok_or(Denial::Registration(
        PublicationCompanionRegistrationStop::OwnerUnavailable,
    ))?;
    let configuration = runtime.configuration_binding();
    let _configuration = configuration.operation();
    let publication = runtime.publication_binding();
    let registry = publication.companion_registry();
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
            let branch = runtime
                .branch_identity(selected.branch_id())
                .map_err(|_| Denial::ForeignSource)?;
            let basis = runtime
                .admit_branch_basis(&branch)
                .map_err(|_| Denial::ForeignSource)?;
            if selected.runtime_instance_id() != runtime.runtime_instance_id()
                || !Arc::ptr_eq(&basis.inner.root, root)
                || !root.managed_source_image.is_readmitted()
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
                runtime: runtime.runtime_instance_id(),
            };
            root.managed_source_image
                .attach_readmitted(execution, selected.branch_id(), &image, || {
                    lookup =
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
    Arc::get_mut(&mut cell.core)
        .expect("unshared Source placement")
        ._placement_memory = memory.clone();
    let image = ManagedSourceRootImage {
        core: Arc::downgrade(&cell.core),
        affinity: cell.core.retain_source_affinity(execution)?,
        image: cell.read_image().image,
        _placement_memory: memory,
    };
    Ok((PreparedManagedSourceCell::new(cell), image))
}
