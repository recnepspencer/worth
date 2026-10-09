use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::history::data::{BranchId, CommitId};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::PositionedRelationalSnapshot;

use super::preflight::{CompanionPreflightStop, PublicationCompanionPreflight};

mod derived_replacement;
pub use derived_replacement::{
    CompanionDerivedImageRetention, CompanionDerivedRootAdmission, CompanionDerivedRootCleanup,
    CompanionDerivedRootCost, CompanionDerivedRootInstalled, CompanionDerivedRootPreparationStop,
    CompanionDerivedRootStopped, PreparedCompanionDerivedRoot,
};

pub(super) struct CompanionRootImage {
    pub(super) root_id: u64,
    pub(super) commit_id: Option<CommitId>,
    pub(super) position: AtomicU64,
    pub(super) payload: Arc<dyn Any + Send + Sync>,
    pub(super) _derived_retention: Option<CompanionDerivedImageRetention>,
}

impl CompanionRootImage {
    pub(super) fn position(&self) -> Option<PatchStreamPosition> {
        match self.position.load(Ordering::Acquire) {
            0 => None,
            value => Some(PatchStreamPosition(value)),
        }
    }
}

pub(super) struct CompanionBranchCellCore {
    pub(super) runtime_instance_id: u64,
    pub(super) registration_generation: u64,
    pub(super) branch_id: BranchId,
    pub(super) current: Mutex<Arc<CompanionRootImage>>,
    pub(super) reservation_live: AtomicBool,
    pub(super) topology_generation: AtomicU64,
}

struct DerivedCellEdit<'a>(&'a CompanionBranchCellCore);

impl Drop for DerivedCellEdit<'_> {
    fn drop(&mut self) {
        self.0.reservation_live.store(false, Ordering::Release);
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionCellEditStop {
    PreflightPending,
    TopologyGenerationChanged,
    TopologyGenerationExhausted,
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
    pub(super) core: Arc<CompanionBranchCellCore>,
    pub(super) observed: CompanionBranchImage<T>,
    pub(super) candidate_id: u64,
    armed: bool,
}

impl<T: Send + Sync + 'static> ReservedCompanionBranchCell<T> {
    pub fn current(&self) -> &CompanionBranchImage<T> {
        &self.observed
    }

    pub(super) fn into_core(mut self) -> Arc<CompanionBranchCellCore> {
        self.armed = false;
        Arc::clone(&self.core)
    }
}

impl<T: Send + Sync + 'static> Drop for ReservedCompanionBranchCell<T> {
    fn drop(&mut self) {
        if self.armed {
            self.core.reservation_live.store(false, Ordering::Release);
        }
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

    fn at_selection(
        runtime_instance_id: u64,
        branch_id: BranchId,
        root_id: u64,
        commit_id: Option<CommitId>,
        position: Option<PatchStreamPosition>,
        registration_generation: u64,
        payload: Arc<T>,
    ) -> Self {
        let image = Arc::new(CompanionRootImage {
            root_id,
            commit_id,
            position: AtomicU64::new(position.map_or(0, |position| position.0)),
            payload,
            _derived_retention: None,
        });
        Self {
            core: Arc::new(CompanionBranchCellCore {
                runtime_instance_id,
                registration_generation,
                branch_id,
                current: Mutex::new(image),
                reservation_live: AtomicBool::new(false),
                topology_generation: AtomicU64::new(1),
            }),
            _payload: PhantomData,
        }
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.core.branch_id
    }

    pub fn read_image(&self) -> CompanionBranchImage<T> {
        let current = self.core.current.lock().unwrap_or_else(|p| p.into_inner());
        let image = Arc::clone(&*current);
        let topology_generation = self.core.topology_generation.load(Ordering::Acquire);
        drop(current);
        let payload = Arc::downcast::<T>(Arc::clone(&image.payload))
            .expect("registered companion cell retains its original payload type");
        CompanionBranchImage {
            image: Arc::clone(&image),
            payload,
            root_id: image.root_id,
            commit_id: image.commit_id,
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
        let prepared = derived_replacement::prepare_admitted(self, expected, prepared, None);
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
        self.core
            .reservation_live
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| CompanionPreflightStop::TopologyPending)?;
        let observed = self.read_image();
        if observed.root_id != context.expected_root_id()
            || observed.commit_id != context.expected_commit_id()
        {
            self.core.reservation_live.store(false, Ordering::Release);
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(ReservedCompanionBranchCell {
            core: Arc::clone(&self.core),
            observed,
            candidate_id: context.candidate_id(),
            armed: true,
        })
    }
}
