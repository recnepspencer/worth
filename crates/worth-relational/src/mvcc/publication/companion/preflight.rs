use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, MutexGuard, TryLockError};

use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::{
    PositionedRelationalSnapshot, RelationalInterruptionBoundary, RelationalInterruptionEvent,
    RelationalOperationControl,
};

use super::cell::{CompanionBranchCellCore, CompanionRootImage};
use super::{CompanionBranchCell, CompanionRegistry, ReservedCompanionBranchCell};

mod observer;
pub use observer::{CompanionPublicationCompletion, CompanionPublicationCompletionObserver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionPreflightBudget {
    pub maximum_work_visits: u64,
    pub maximum_preparation_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionPreflightStop {
    TopologyPending,
    SelectedSourceMismatch,
    SelectedPositionUnavailable {
        commit_id: CommitId,
    },
    ForeignCell,
    RegistrationChanged,
    CellCapacityExhausted {
        maximum_bytes: u64,
    },
    WorkExhausted {
        required: u64,
        maximum: u64,
    },
    WorkCounterOverflow,
    PreparationMemoryExhausted {
        required: u64,
        maximum: u64,
    },
    PreparationMemoryCounterOverflow,
    RetainedCompanionCapacityExhausted {
        requested: u64,
        retained: u64,
        maximum: u64,
    },
    Interrupted(RelationalInterruptionEvent),
}

/// Query-owned preparation runs here, before Relational's irreversible cutover.
pub trait RelationalPublicationCompanion: Debug + Send + Sync {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop>;
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CandidateCompanionBinding {
    pub(crate) runtime_instance_id: u64,
    pub(crate) registration_generation: u64,
    pub(crate) candidate_id: u64,
    pub(crate) branch_id: BranchId,
    pub(crate) expected_root_id: u64,
    pub(crate) expected_commit_id: Option<CommitId>,
    pub(crate) expected_position: Option<PatchStreamPosition>,
    pub(crate) next_root_id: u64,
    pub(crate) next_commit_id: CommitId,
}

pub struct PublicationCompanionPreflight<'a> {
    pub(crate) binding: CandidateCompanionBinding,
    pub(crate) envelope: &'a CanonicalCommitEnvelope,
    pub(crate) control: &'a RelationalOperationControl,
    pub(crate) budget: CompanionPreflightBudget,
    pub(crate) cells: Arc<CompanionRegistry>,
    charged_work: u64,
    prepared_bytes: u64,
}

impl<'a> PublicationCompanionPreflight<'a> {
    pub(crate) fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
        budget: CompanionPreflightBudget,
        cells: Arc<CompanionRegistry>,
    ) -> Self {
        Self {
            binding,
            envelope,
            control,
            budget,
            cells,
            charged_work: 0,
            prepared_bytes: 0,
        }
    }

    pub fn canonical_commit(&self) -> &CanonicalCommitEnvelope {
        self.envelope
    }
    pub fn runtime_instance_id(&self) -> u64 {
        self.binding.runtime_instance_id
    }
    pub fn branch_id(&self) -> &BranchId {
        &self.binding.branch_id
    }
    pub fn expected_root_id(&self) -> u64 {
        self.binding.expected_root_id
    }
    pub fn expected_commit_id(&self) -> Option<CommitId> {
        self.binding.expected_commit_id
    }
    pub const fn expected_position(&self) -> Option<PatchStreamPosition> {
        self.binding.expected_position
    }
    pub(crate) fn candidate_id(&self) -> u64 {
        self.binding.candidate_id
    }

    pub fn checkpoint(&self) -> Result<(), CompanionPreflightStop> {
        self.control
            .observe(RelationalInterruptionBoundary::PublicationPreflight)
            .map_or(Ok(()), |event| {
                Err(CompanionPreflightStop::Interrupted(event))
            })
    }

    pub fn claim_work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        self.checkpoint()?;
        let required = self
            .charged_work
            .checked_add(visits)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        if required > self.budget.maximum_work_visits {
            return Err(CompanionPreflightStop::WorkExhausted {
                required,
                maximum: self.budget.maximum_work_visits,
            });
        }
        self.charged_work = required;
        Ok(())
    }

    pub fn claim_bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.checkpoint()?;
        let required = self
            .prepared_bytes
            .checked_add(bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        if required > self.budget.maximum_preparation_bytes {
            return Err(CompanionPreflightStop::PreparationMemoryExhausted {
                required,
                maximum: self.budget.maximum_preparation_bytes,
            });
        }
        self.prepared_bytes = required;
        Ok(())
    }

    /// Mint an initial cell only for this candidate's currently selected root.
    pub fn mint_branch_cell<T: Send + Sync + 'static>(
        &mut self,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, CompanionPreflightStop> {
        if selected.runtime_instance_id() != self.binding.runtime_instance_id
            || selected.branch_id() != &self.binding.branch_id
            || selected.root_id() != self.binding.expected_root_id
            || selected.commit_id() != self.binding.expected_commit_id
            || selected.position() != self.binding.expected_position
        {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        self.claim_work(1)?;
        self.claim_bytes(
            arc_allocation_bound::<CompanionRootImage>()
                .saturating_add(arc_allocation_bound::<CompanionBranchCellCore>()),
        )?;
        let retention = self.cells.reserve_cell().map_err(|stop| match stop {
            super::PublicationCompanionRegistrationStop::CellCapacityExhausted {
                maximum_bytes,
            } => CompanionPreflightStop::CellCapacityExhausted { maximum_bytes },
            _ => CompanionPreflightStop::RegistrationChanged,
        })?;
        Ok(CompanionBranchCell::new(
            selected,
            self.binding.registration_generation,
            initial,
            retention,
        ))
    }

    /// Mint the first cell from the exact owner-admitted selected root.
    /// The caller supplies only its prepared Query payload, never source identity.
    pub fn mint_selected_branch_cell<T: Send + Sync + 'static>(
        &mut self,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, CompanionPreflightStop> {
        self.claim_work(1)?;
        self.claim_bytes(
            arc_allocation_bound::<CompanionRootImage>()
                .saturating_add(arc_allocation_bound::<CompanionBranchCellCore>()),
        )?;
        let retention = self.cells.reserve_cell().map_err(|stop| match stop {
            super::PublicationCompanionRegistrationStop::CellCapacityExhausted {
                maximum_bytes,
            } => CompanionPreflightStop::CellCapacityExhausted { maximum_bytes },
            _ => CompanionPreflightStop::RegistrationChanged,
        })?;
        Ok(CompanionBranchCell::new_selected(
            &self.binding,
            initial,
            retention,
        ))
    }

    pub(super) fn validate_cell(
        &self,
        cell: &CompanionBranchCellCore,
    ) -> Result<(), CompanionPreflightStop> {
        if cell.runtime_instance_id != self.binding.runtime_instance_id
            || cell.branch_id != self.binding.branch_id
        {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        if cell.registration_generation != self.binding.registration_generation {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        Ok(())
    }

    pub fn seal_replacement<T: Send + Sync + 'static>(
        &mut self,
        reserved: ReservedCompanionBranchCell<T>,
        prepared: Arc<T>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.validate_cell(&reserved.core)?;
        if reserved.candidate_id != self.binding.candidate_id
            || reserved.current().root_id() != self.binding.expected_root_id
            || reserved.current().commit_id() != self.binding.expected_commit_id
        {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        self.claim_work(1)?;
        self.claim_bytes(arc_allocation_bound::<CompanionRootImage>())?;
        if reserved.core.topology_generation.load(Ordering::Acquire)
            != reserved.current().topology_generation()
        {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        let next_topology_generation = reserved
            .current()
            .topology_generation()
            .checked_add(1)
            .ok_or(CompanionPreflightStop::RegistrationChanged)?;
        let next = Arc::new(CompanionRootImage {
            root_id: self.binding.next_root_id,
            commit_id: Some(self.binding.next_commit_id),
            position: AtomicU64::new(0),
            payload: prepared,
            _derived_retention: None,
        });
        let pinned_image = Arc::clone(&reserved.current().image);
        let core = reserved.into_core();
        Ok(PreparedPublicationCompanionEffect {
            binding: self.binding.clone(),
            pinned_topology_generation: next_topology_generation - 1,
            pinned_image,
            cell: core,
            next,
            retired: None,
            next_topology_generation,
            completion: None,
        })
    }
}

/// Non-cloneable, candidate-bound successor image. Only Relational installs it.
pub struct PreparedPublicationCompanionEffect {
    binding: CandidateCompanionBinding,
    pinned_topology_generation: u64,
    pinned_image: Arc<CompanionRootImage>,
    cell: Arc<CompanionBranchCellCore>,
    next: Arc<CompanionRootImage>,
    retired: Option<Arc<CompanionRootImage>>,
    next_topology_generation: u64,
    completion: Option<CompanionPublicationCompletionObserver>,
}

impl PreparedPublicationCompanionEffect {
    /// The caller funds this scheduling observer before native publication.
    /// It never changes the authority or selected-image checks of the effect.
    pub fn attach_completion_observer(
        &mut self,
        context: &mut PublicationCompanionPreflight<'_>,
        retained: Arc<dyn Send + Sync>,
    ) -> Result<CompanionPublicationCompletionObserver, CompanionPreflightStop> {
        assert!(
            self.completion.is_none(),
            "one observer per prepared cutover"
        );
        context.claim_work(2)?;
        context.claim_bytes(CompanionPublicationCompletionObserver::retained_bytes())?;
        let observer = CompanionPublicationCompletionObserver::new(retained);
        self.completion = Some(observer.clone());
        Ok(observer)
    }

    pub(crate) fn matches(
        &self,
        binding: &CandidateCompanionBinding,
    ) -> Result<(), CompanionPreflightStop> {
        if self.binding != *binding
            || self.cell.registration_generation != binding.registration_generation
        {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        if !self.cell.reservation_live.load(Ordering::Acquire)
            || self.cell.topology_generation.load(Ordering::Acquire)
                != self.pinned_topology_generation
            || self.pinned_topology_generation.checked_add(1) != Some(self.next_topology_generation)
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(())
    }

    /// Acquire the fixed swap cell before any branch movement.
    pub(crate) fn enter_cutover(
        &mut self,
    ) -> Result<PreparedCompanionCutover<'_>, CompanionPreflightStop> {
        let current = match self.cell.current.try_lock() {
            Ok(current) => current,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(CompanionPreflightStop::TopologyPending),
        };
        if self.cell.topology_generation.load(Ordering::Acquire) != self.pinned_topology_generation
            || !Arc::ptr_eq(&*current, &self.pinned_image)
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(PreparedCompanionCutover {
            cell: &self.cell,
            next: &self.next,
            retired: &mut self.retired,
            next_topology_generation: self.next_topology_generation,
            current,
            completion: self.completion.as_ref(),
        })
    }
}

pub(crate) struct PreparedCompanionCutover<'a> {
    cell: &'a Arc<CompanionBranchCellCore>,
    next: &'a Arc<CompanionRootImage>,
    retired: &'a mut Option<Arc<CompanionRootImage>>,
    next_topology_generation: u64,
    current: MutexGuard<'a, Arc<CompanionRootImage>>,
    completion: Option<&'a CompanionPublicationCompletionObserver>,
}

impl PreparedCompanionCutover<'_> {
    /// No callbacks, allocation, failure, or retired-payload Drop occur here.
    pub(crate) fn install_at(&mut self, position: PatchStreamPosition) {
        self.next.position.store(position.0, Ordering::Release);
        *self.retired = Some(std::mem::replace(&mut *self.current, Arc::clone(self.next)));
        self.cell
            .topology_generation
            .store(self.next_topology_generation, Ordering::Release);
        if let Some(completion) = self.completion {
            completion.installed();
        }
    }
}

pub(super) const fn arc_allocation_bound<T>() -> u64 {
    (std::mem::size_of::<T>() + 2 * std::mem::size_of::<usize>() + std::mem::align_of::<T>()) as u64
}

impl Drop for PreparedPublicationCompanionEffect {
    fn drop(&mut self) {
        if let Some(completion) = &self.completion {
            completion.abort_uninstalled();
        }
        self.cell.reservation_live.store(false, Ordering::Release);
    }
}
