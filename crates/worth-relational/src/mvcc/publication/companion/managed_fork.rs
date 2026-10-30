//! A genuine fork's target association is prepared before its durable transition.
use super::cell::publication_admission::PreparedManagedForkCell;
use super::cell::{CompanionBranchCellCore, CompanionRootImage};
use super::{CompanionBranchCell, CompanionBranchCellSlot, CompanionPreflightStop as Stop};
use crate::branch::{
    RelationalBranchRoot, RelationalBranchRootIdentityIssuer, RelationalForkDenial,
};
use crate::history::data::BranchId;
use crate::validation::candidate_source::ManagedIndexBranchImage;
use std::sync::Arc;

/// Lookup-only custody for one actual prepared target. It grants no Source READ.
pub struct ManagedIndexSourceForkPreparation<'a> {
    cell: &'a PreparedManagedForkCell,
    branch: &'a BranchId,
    runtime: u64,
}
impl ManagedIndexSourceForkPreparation<'_> {
    pub fn target_branch(&self) -> &BranchId {
        self.branch
    }
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime
    }
    pub fn lookup_slot(&self) -> CompanionBranchCellSlot<ManagedIndexBranchImage> {
        self.cell.lookup_slot()
    }
}

pub(crate) struct PreparedManagedIndexSourceFork {
    cell: PreparedManagedForkCell,
}
impl PreparedManagedIndexSourceFork {
    pub(crate) fn prepare(
        source: &RelationalBranchRoot,
        source_branch: &BranchId,
        target_branch: &BranchId,
        runtime: u64,
        generation: u64,
        participant: &dyn super::RelationalPublicationCompanion,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        issuer: &RelationalBranchRootIdentityIssuer,
        lookup_cleanup: &mut Option<Box<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(Arc<RelationalBranchRoot>, Self), RelationalForkDenial> {
        let denied = RelationalForkDenial::ManagedSource;
        crate::authority::commit::pipeline::consult_stage_control(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| denied(Stop::Execution(cause.into())))?;
        let original = source
            .managed_source_image
            .get()
            .ok_or_else(|| denied(Stop::ManagedSourceNotPrepared))?;
        let owner = original
            .core
            .upgrade()
            .ok_or_else(|| denied(Stop::ForeignCell))?;
        if owner.runtime_instance_id != runtime
            || owner.branch_id != *source_branch
            || owner.registration_generation != generation
            || original.image.root_id != source.id()
            || original.image.commit_id != source.commit_id()
        {
            return Err(denied(Stop::SelectedSourceMismatch));
        }
        let source_payload = Arc::clone(&original.image.payload)
            .downcast::<ManagedIndexBranchImage>()
            .map_err(|_| denied(Stop::ForeignCell))?;
        // Pay before constructing the header, core, image, admission and grant
        // Arc. These are one fork resource, sharing no source reader's allowance.
        let bytes = metadata_bytes(target_branch)
            .ok_or_else(|| denied(Stop::PreparationMemoryCounterOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| denied(Stop::Execution(cause.into())))?
            .map(Arc::new);
        // Fresh cache slot for the branch; immutable directory cells and their
        // payloads remain O(1) temporal shares until either image edits its slot.
        let payload = ManagedIndexBranchImage::retain_header(execution, (*source_payload).clone())
            .map_err(denied)?;
        let header = source
            .prepare_managed_fork_header(issuer)
            .map_err(|_| RelationalForkDenial::RootIdentityExhausted)?;
        let mut cell = CompanionBranchCell::at_selection(
            runtime,
            target_branch.clone(),
            header.id(),
            header.commit_id(),
            // The fork transition has a later stream position. Material still
            // belongs to this selected source commit's actual canonical position.
            original.image.position(),
            generation,
            payload,
        );
        Arc::get_mut(&mut cell.core)
            .expect("fresh unshared fork core")
            ._fork_memory = memory.clone();
        let image = cell.read_image();
        let affinity = cell
            .core
            .retain_source_affinity(execution)
            .map_err(denied)?;
        header
            .managed_source_image
            .set(super::ManagedSourceRootImage {
                core: Arc::downgrade(&cell.core),
                affinity,
                image: Arc::clone(&image.image),
                _fork_memory: memory,
            })
            .map_err(|_| denied(Stop::SelectedSourceMismatch))?;
        let cell = PreparedManagedForkCell::new(cell);
        let context = ManagedIndexSourceForkPreparation {
            cell: &cell,
            branch: target_branch,
            runtime,
        };
        // The fork caller declared this owner before its Native guards. A
        // stopped preparation must not run a participant's Drop under them.
        *lookup_cleanup = Some(
            participant
                .prepare_managed_source_fork(execution, &context)
                .map_err(denied)?,
        );
        crate::authority::commit::pipeline::consult_stage_control(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| denied(Stop::Execution(cause.into())))?;
        Ok((header, Self { cell }))
    }

    /// Called only inside the durable fork's infallible target cutover.
    pub(crate) fn installed(&self) {
        self.cell.installed();
    }
}

fn metadata_bytes(branch: &BranchId) -> Option<u64> {
    let fixed = [
        std::mem::size_of::<RelationalBranchRoot>(),
        std::mem::size_of::<CompanionBranchCellCore>(),
        std::mem::size_of::<CompanionRootImage>(),
        PreparedManagedForkCell::metadata_bytes(),
        std::mem::size_of::<worth_execution::ExecutionMemoryReservation>(),
    ]
    .into_iter()
    .try_fold(0_usize, |total, bytes| {
        total.checked_add(bytes.checked_add(2 * std::mem::size_of::<usize>())?)
    })?;
    u64::try_from(fixed.checked_add(branch.0.len().checked_mul(4)?)?).ok()
}
