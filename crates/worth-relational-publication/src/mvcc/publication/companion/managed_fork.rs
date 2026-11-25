//! A genuine fork's target association is prepared before its durable transition.
use super::cell::publication_admission::PreparedManagedSourceCell;
use super::CompanionBranchCell;
use super::CompanionBranchCellSlot;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::fork::RelationalForkDenial;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::{
    CompanionBranchCellCore, CompanionRootImage,
};
use worth_relational_branch::internal_port::branch::root::identity::RelationalBranchRootIdentityIssuer;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::validation::candidate_source::image::publication::ManagedIndexBranchImageOwnerPort as _;
use worth_relational_branch::internal_port::validation::candidate_source::image::ManagedIndexBranchImage;
use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_contracts::internal_port::mvcc::publication::companion::preflight_stop::CompanionPreflightStop as Stop;

/// Lookup-only custody for a Source cell awaiting Native placement.
pub struct ManagedIndexSourceCellPlacement<'a> {
    pub(super) cell: &'a PreparedManagedSourceCell,
    pub(super) branch: &'a BranchId,
    pub(super) runtime: u64,
}
impl ManagedIndexSourceCellPlacement<'_> {
    pub fn branch(&self) -> &BranchId {
        self.branch
    }
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime
    }
    pub fn lookup_slot(&self) -> CompanionBranchCellSlot<ManagedIndexBranchImage> {
        self.cell.lookup_slot()
    }
}

pub struct PreparedManagedIndexSourceFork {
    cell: PreparedManagedSourceCell,
}
impl PreparedManagedIndexSourceFork {
    pub fn prepare(
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
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| denied(Stop::Execution(cause.into())))?;
        let original = source
            .managed_source_image
            .for_branch(source_branch)
            .ok_or_else(|| denied(Stop::ManagedSourceNotPrepared))?;
        let owner = original
            .serving_cell()
            .ok_or_else(|| denied(Stop::ForeignCell))?;
        if owner.runtime_instance_id() != runtime
            || owner.branch_id() != source_branch
            || owner.registration_generation() != generation
            || original.retained_image().root_id() != source.id()
            || original.retained_image().commit_id() != source.commit_id()
        {
            return Err(denied(Stop::SelectedSourceMismatch));
        }
        let source_payload = original
            .retained_image()
            .retain_payload::<ManagedIndexBranchImage>()
            .ok_or_else(|| denied(Stop::ForeignCell))?;
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
            original.retained_image().position(),
            generation,
            payload,
        );
        CompanionBranchCellCore::bind_unshared_placement_funding(&mut cell.core, memory);
        cell.core
            .retain_source_affinity(execution)
            .map_err(denied)?;
        header
            .managed_source_image
            .set(
                worth_relational_branch::internal_port::branch::projection_view::managed_source::ManagedSourceRootImage::capture_current(&cell.core)
                    .expect("fresh fork cell retained its original affinity"),
            )
            .map_err(|_| denied(Stop::SelectedSourceMismatch))?;
        let cell = PreparedManagedSourceCell::new(cell);
        let context = ManagedIndexSourceCellPlacement {
            cell: &cell,
            branch: target_branch,
            runtime,
        };
        // The fork caller declared this owner before its Native guards. A
        // stopped preparation must not run a participant's Drop under them.
        *lookup_cleanup = Some(
            participant
                .prepare_managed_source_placement(execution, &context)
                .map_err(denied)?,
        );
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| denied(Stop::Execution(cause.into())))?;
        Ok((header, Self { cell }))
    }

    /// Called only inside the durable fork's infallible target cutover.
    pub fn installed(&self) {
        self.cell.installed();
    }
}

fn metadata_bytes(branch: &BranchId) -> Option<u64> {
    let fixed = [
        std::mem::size_of::<RelationalBranchRoot>(),
        std::mem::size_of::<CompanionBranchCellCore>(),
        std::mem::size_of::<CompanionRootImage>(),
        PreparedManagedSourceCell::metadata_bytes(),
        std::mem::size_of::<worth_execution::ExecutionMemoryReservation>(),
    ]
    .into_iter()
    .try_fold(0_usize, |total, bytes| {
        total.checked_add(bytes.checked_add(2 * std::mem::size_of::<usize>())?)
    })?;
    u64::try_from(fixed.checked_add(branch.0.len().checked_mul(4)?)?).ok()
}
