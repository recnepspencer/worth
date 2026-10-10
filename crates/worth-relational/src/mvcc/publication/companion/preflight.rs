use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::{
    RelationalInterruptionBoundary, RelationalInterruptionEvent, RelationalOperationControl,
};

use super::cell::publication_admission::PublicationCellAdmission;
use super::cell::{CompanionBranchCellCore, CompanionRootImage};
use super::{PreparedCompanionBranchCell, ReservedCompanionBranchCell};
mod cutover;
pub use cutover::PreparedPublicationCompanionEffect;

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
    charged_work: u64,
    prepared_bytes: u64,
}

impl<'a> PublicationCompanionPreflight<'a> {
    pub(crate) fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
        budget: CompanionPreflightBudget,
    ) -> Self {
        Self {
            binding,
            envelope,
            control,
            budget,
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

    /// Prepare an unpublished cell from the candidate expectation, verified only at cutover.
    /// The caller supplies only its prepared Query payload, never source identity.
    pub fn mint_selected_branch_cell<T: Send + Sync + 'static>(
        &mut self,
        initial: Arc<T>,
    ) -> Result<PreparedCompanionBranchCell<T>, CompanionPreflightStop> {
        self.claim_work(1)?;
        self.claim_bytes(
            arc_allocation_bound::<CompanionRootImage>()
                .saturating_add(arc_allocation_bound::<CompanionBranchCellCore>())
                .saturating_add(arc_allocation_bound::<PublicationCellAdmission>()),
        )?;
        Ok(PreparedCompanionBranchCell::new_selected(
            &self.binding,
            initial,
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
        mut reserved: ReservedCompanionBranchCell<T>,
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
        let publication_admission = reserved.publication_admission.take();
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
            publication_admission,
        })
    }
}

pub(super) const fn arc_allocation_bound<T>() -> u64 {
    (std::mem::size_of::<T>() + 2 * std::mem::size_of::<usize>() + std::mem::align_of::<T>()) as u64
}
