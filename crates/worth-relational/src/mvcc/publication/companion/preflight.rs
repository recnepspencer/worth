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
mod cell_swap;
mod cutover;
pub use cutover::PreparedPublicationCompanionEffect;

mod observer;
pub use observer::{CompanionPublicationCompletion, CompanionPublicationCompletionObserver};

/// The original reason native companion admission or preparation stopped.
///
/// Execution retains the actual request refusal; counter overflow carries no
/// inferred requirement. Structural variants name
/// pending topology, a mismatched source or position, a foreign cell, or changed
/// registration, without implying resource exhaustion. Execution and interruption
/// variants retain the native cause and, when supplied, its boundary.
///
/// This is diagnostic data, not publication, currentness or cleanup authority.
/// The receiving owner supplies the reporting stage and decides recovery;
/// a stop alone neither identifies that stage nor promises a successful retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionPreflightStop {
    Execution(crate::execution::RelationalExecutionDenialCause),
    TopologyPending,
    SelectedSourceMismatch,
    SelectedPositionUnavailable { commit_id: CommitId },
    ForeignCell,
    RegistrationChanged,
    WorkCounterOverflow,
    PreparationMemoryCounterOverflow,
    Interrupted(RelationalInterruptionEvent),
}

/// Query-owned preparation runs here, before Relational's irreversible cutover.
pub trait RelationalPublicationCompanion: Debug + Send + Sync {
    /// Runs on the actual applied draft before Native's FIRST invariant. It
    /// prepares only installed derived Source roots, never validates a rule.
    /// The default consults the original request's validation stage before refusal.
    fn prepare_managed_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _source: &crate::validation::candidate_source::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<crate::validation::candidate_source::PreparedManagedIndexSourceRoot>,
        crate::validation::candidate_source::ManagedIndexSourcePreparationDenial,
    > {
        crate::authority::commit::pipeline::consult_stage_control(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )
        .map_err(|cause| {
            crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
        })?;
        Err(crate::validation::candidate_source::ManagedIndexSourcePreparationDenial::MissingParticipant)
    }

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
}

impl<'a> PublicationCompanionPreflight<'a> {
    pub(crate) fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
    ) -> Self {
        Self {
            binding,
            envelope,
            control,
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

    /// Prepare an unpublished cell from the candidate expectation, verified only at cutover.
    /// The caller supplies only its prepared Query payload, never source identity.
    pub fn mint_selected_branch_cell<T: Send + Sync + 'static>(
        &mut self,
        initial: Arc<T>,
    ) -> Result<PreparedCompanionBranchCell<T>, CompanionPreflightStop> {
        self.checkpoint()?;
        self.checkpoint()?;
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
        self.checkpoint()?;
        self.checkpoint()?;
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
        });
        let pinned_image = Arc::clone(&reserved.current().image);
        let publication_admission = reserved.publication_admission.take();
        let core = reserved.into_core();
        Ok(PreparedPublicationCompanionEffect {
            binding: self.binding.clone(),
            primary: Some(cell_swap::PreparedCellSwap {
                pinned_topology_generation: next_topology_generation - 1,
                pinned_image,
                cell: core,
                next,
                retired: None,
                next_topology_generation,
                publication_admission,
            }),
            managed: None,
            completion: None,
        })
    }

    /// The existing required participant publishes exactly two fixed images:
    /// consumed-Source marks and managed Source/root state. This is not a list
    /// of independently registered callbacks or extensible publication effects.
    /// Both reservations must belong to this candidate and its selected source.
    /// A refusal releases both reservations and unpublished replacements, including
    /// a first replacement prepared before the second cell refuses. Dropping the
    /// sealed effect also releases its unpublished backing. The publication port
    /// checks the original request and installs both images at the graph's one
    /// Native-issued publication position; callers cannot supply that position.
    pub fn seal_managed_source_replacement<
        Marks: Send + Sync + 'static,
        Managed: Send + Sync + 'static,
    >(
        &mut self,
        marks: ReservedCompanionBranchCell<Marks>,
        next_marks: Arc<Marks>,
        managed: ReservedCompanionBranchCell<Managed>,
        next_managed: Arc<Managed>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        if Arc::ptr_eq(&marks.core, &managed.core) {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let mut effect = self.seal_replacement(marks, next_marks)?;
        let mut source = self.seal_replacement(managed, next_managed)?;
        effect.managed = source.primary.take();
        Ok(effect)
    }
}
