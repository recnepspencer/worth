use worth_relational_contracts::internal_port as contracts;
pub mod managed_source;
pub mod read_condition;
pub use managed_source::ManagedIndexSourceRebuild;
pub(in crate::internal_port::mvcc) use managed_source::ManagedSourceReadConditions;
pub(in crate::internal_port::mvcc) use read_condition::CompanionReadCondition;
use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;
pub use worth_relational_branch::internal_port::branch::projection_view::managed_source::ManagedIndexSourceObservation;
#[cfg(test)]
pub(crate) use worth_relational_branch::internal_port::branch::projection_view::managed_source::ManagedSourceRootImage;

use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::positioned_snapshot::PositionedRelationalSnapshot;

use super::preflight::PublicationCompanionPreflight;
use contracts::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;

pub mod publication_admission;
pub use publication_admission::{CompanionBranchCellSlot, PreparedCompanionBranchCell};
pub mod derived_replacement;
pub mod public_cache;
pub use derived_replacement::{
    CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, PreparedCompanionDerivedRoot,
};
pub use worth_relational_branch::internal_port::branch::projection_view::public_cache::{
    ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement,
};

pub(in crate::internal_port::mvcc) use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionBranchCellCore;
pub(in crate::internal_port::mvcc) use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionRootImage;

pub use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionCellEditStop;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionSlotReservation;

/// Typed read and derived-edit access to one registered branch companion root.
pub struct CompanionBranchCell<T: Send + Sync + 'static> {
    pub(super) core: Arc<CompanionBranchCellCore>,
    _payload: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Clone for CompanionBranchCell<T> {
    fn clone(&self) -> Self {
        Self {
            core: Arc::clone(&self.core),
            _payload: PhantomData,
        }
    }
}

pub struct CompanionBranchImage<T: Send + Sync + 'static> {
    pub(super) image: Arc<CompanionRootImage>,
    pub(crate) payload: Arc<T>,
    pub(crate) root_id: u64,
    pub(crate) commit_id: Option<CommitId>,
    pub(crate) position: Option<PatchStreamPosition>,
    pub(crate) topology_generation: u64,
}

impl<T: Send + Sync + 'static> CompanionBranchImage<T> {
    pub fn payload(&self) -> &Arc<T> {
        &self.payload
    }
    pub const fn root_id(&self) -> u64 {
        self.root_id
    }
    pub const fn commit_id(&self) -> Option<CommitId> {
        self.commit_id
    }
    pub const fn position(&self) -> Option<PatchStreamPosition> {
        self.position
    }
    pub const fn topology_generation(&self) -> u64 {
        self.topology_generation
    }
}

/// The sole live topology pin for a candidate's branch companion cell.
pub struct ReservedCompanionBranchCell<T: Send + Sync + 'static> {
    reservation: CompanionSlotReservation,
    pub(super) observed: CompanionBranchImage<T>,
    pub(super) candidate_id: u64,
    pub(super) publication_admission: Option<Arc<publication_admission::PublicationCellAdmission>>,
}

impl<T: Send + Sync + 'static> ReservedCompanionBranchCell<T> {
    pub fn current(&self) -> &CompanionBranchImage<T> {
        &self.observed
    }

    pub(super) fn core(&self) -> &Arc<CompanionBranchCellCore> {
        self.reservation.cell()
    }

    pub(super) fn into_reservation(self) -> CompanionSlotReservation {
        self.reservation
    }
}

impl<T: Send + Sync + 'static> CompanionBranchCell<T> {
    pub(super) fn new(
        snapshot: &PositionedRelationalSnapshot,
        registration_generation: u64,
        payload: Arc<T>,
    ) -> Self {
        Self::at_selection(
            snapshot.runtime_instance_id(),
            snapshot.branch_id().clone(),
            snapshot.root_id(),
            snapshot.commit_id(),
            snapshot.position(),
            registration_generation,
            payload,
        )
    }

    pub(super) fn new_selected(
        binding: &super::preflight::CandidateCompanionBinding,
        payload: Arc<T>,
    ) -> Self {
        Self::at_selection(
            binding.runtime_instance_id,
            binding.branch_id.clone(),
            binding.expected_root_id,
            binding.expected_commit_id,
            binding.expected_position,
            binding.registration_generation,
            payload,
        )
    }

    pub(super) fn at_selection(
        runtime_instance_id: u64,
        branch_id: BranchId,
        root_id: u64,
        commit_id: Option<CommitId>,
        position: Option<PatchStreamPosition>,
        registration_generation: u64,
        payload: Arc<T>,
    ) -> Self {
        let image = CompanionRootImage::new(root_id, commit_id, position, payload);
        Self {
            core: CompanionBranchCellCore::new(
                runtime_instance_id,
                branch_id,
                registration_generation,
                image,
            ),
            _payload: PhantomData,
        }
    }

    pub fn branch_id(&self) -> &BranchId {
        self.core.branch_id()
    }

    pub fn read_image(&self) -> CompanionBranchImage<T> {
        let current = self.core.read_image();
        let image = Arc::clone(current.image());
        let topology_generation = self.core.generation();
        drop(current);
        let payload = image.retain_payload::<T>()
            .expect(/* Proof: Registration binds this cell to T and stores only Arc<T> through that typed handle; no untyped payload replacement door exists. */ "registered companion cell retains its original payload type");
        CompanionBranchImage {
            image: Arc::clone(&image),
            payload,
            root_id: image.root_id(),
            commit_id: image.commit_id(),
            position: image.position(),
            topology_generation,
        }
    }

    /// Publish an already-built derived root at the same source position.
    pub fn replace_derived_root_at_same_position(
        &self,
        expected: CompanionBranchImage<T>,
        prepared: Arc<T>,
    ) -> Result<CompanionBranchImage<T>, CompanionCellEditStop> {
        let prepared = derived_replacement::prepare_admitted(self, expected, prepared);
        let installed = prepared.install().map_err(|stopped| stopped.reason())?;
        let (image, cleanup) = installed.into_parts();
        drop(cleanup);
        Ok(image)
    }

    pub fn reserve_preflight(
        &self,
        context: &PublicationCompanionPreflight<'_>,
    ) -> Result<ReservedCompanionBranchCell<T>, CompanionPreflightStop> {
        context.validate_cell(&self.core)?;
        let reservation = self
            .core
            .reserve_for_publication()
            .ok_or(CompanionPreflightStop::TopologyPending)?;
        let observed = self.read_image();
        if observed.root_id != context.expected_root_id()
            || observed.commit_id != context.expected_commit_id()
        {
            drop(reservation);
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(ReservedCompanionBranchCell {
            reservation,
            observed,
            candidate_id: context.candidate_id(),
            publication_admission: None,
        })
    }
}

#[doc(hidden)]
pub trait CompanionBranchCellOwnerPort<T: Send + Sync + 'static>: Sized {
    fn new(
        snapshot: &PositionedRelationalSnapshot,
        registration_generation: u64,
        payload: Arc<T>,
    ) -> Self;
}

impl<T: Send + Sync + 'static> CompanionBranchCellOwnerPort<T> for CompanionBranchCell<T> {
    fn new(
        snapshot: &PositionedRelationalSnapshot,
        registration_generation: u64,
        payload: Arc<T>,
    ) -> Self {
        Self::new(snapshot, registration_generation, payload)
    }
}

#[doc(hidden)]
pub trait CompanionBranchCellFixturePort {
    fn companion_core_for_test(&self) -> &Arc<CompanionBranchCellCore>;
}
impl<T: Send + Sync + 'static> CompanionBranchCellFixturePort for CompanionBranchCell<T> {
    fn companion_core_for_test(&self) -> &Arc<CompanionBranchCellCore> {
        &self.core
    }
}
