use std::fmt::Debug;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::transfer::PreparedCompanionSlotSwap;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary;
use contracts::runtime::operation_control::RelationalOperationControl;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use super::cell::publication_admission::PublicationCellAdmission;
use super::{PreparedCompanionBranchCell, ReservedCompanionBranchCell};
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::CompanionBranchCellCore;
pub mod cell_swap;
pub mod cutover;
pub mod managed_source;
pub use cutover::PreparedPublicationCompanionEffect;

pub mod observer;
pub use observer::{CompanionPublicationCompletion, CompanionPublicationCompletionObserver};

pub use contracts::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;

/// Query-owned preparation runs here, before Relational's irreversible cutover.
pub trait RelationalPublicationCompanion: Debug + Send + Sync {
    /// Stage only the lookup for Native's pending Source cell.
    /// The returned owner removes its pending metadata on refusal; it carries
    /// no permission to admit the target or to read Source rows.
    fn prepare_managed_source_placement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _placement: &super::ManagedIndexSourceCellPlacement<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, CompanionPreflightStop> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        )
        .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        Err(CompanionPreflightStop::ManagedSourceNotPrepared)
    }
    /// Runs on the actual applied draft before Native's FIRST invariant. It
    /// prepares only installed derived Source roots, never validates a rule.
    /// The default consults the original request's validation stage before refusal.
    fn prepare_managed_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _source: &mutation::validation::candidate_source::preparation::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<mutation::validation::candidate_source::preparation::PreparedManagedIndexSourceRoot>,
        contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial,
    > {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )
        .map_err(|cause| {
            contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(
                cause.into(),
            )
        })?;
        Err(contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial::MissingParticipant)
    }

    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop>;
}

#[derive(Clone, PartialEq, Eq)]
pub struct CandidateCompanionBinding {
    pub runtime_instance_id: u64,
    pub registration_generation: u64,
    pub candidate_id: u64,
    pub branch_id: BranchId,
    pub expected_root_id: u64,
    pub expected_commit_id: Option<CommitId>,
    pub expected_position: Option<PatchStreamPosition>,
    pub next_root_id: u64,
    pub next_commit_id: CommitId,
}

pub struct PublicationCompanionPreflight<'a> {
    pub(crate) binding: CandidateCompanionBinding,
    pub(crate) envelope: &'a CanonicalCommitEnvelope,
    pub(crate) control: &'a RelationalOperationControl,
    sources: mutation::validation::candidate_source::publication::ManagedIndexSourcePublication<'a>,
    pub(crate) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
}

impl<'a> PublicationCompanionPreflight<'a> {
    pub(crate) fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
        source: &'a mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
        execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Result<Self, CompanionPreflightStop> {
        if source.proposed_version() != envelope.commit.version_id {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let sources = source.prepare_publication_loan(
            mutation::validation::candidate_source::publication::ManagedSourcePublicationSelection {
                runtime_instance_id: binding.runtime_instance_id,
                branch_id: &binding.branch_id,
                expected_root_id: binding.expected_root_id,
                registration_generation: binding.registration_generation,
            },
        )?;
        Ok(Self {
            binding,
            envelope,
            control,
            sources,
            execution,
        })
    }

    /// The exact payloads prepared before this candidate's first dependent rule.
    /// Publication cannot rebuild them or turn their logical share into READ.
    pub fn managed_sources(
        &self,
    ) -> &mutation::validation::candidate_source::publication::ManagedIndexSourcePublication<'a>
    {
        &self.sources
    }

    pub fn canonical_commit(&self) -> &CanonicalCommitEnvelope {
        self.envelope
    }
    /// Borrow this publication's original execution for derived backing admission.
    pub fn execution_request(&self) -> worth_execution::ExecutionRequest<'_, '_> {
        *self.execution
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
        if cell.runtime_instance_id() != self.binding.runtime_instance_id
            || cell.branch_id() != &self.binding.branch_id
        {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        if cell.registration_generation() != self.binding.registration_generation {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        Ok(())
    }

    pub fn seal_replacement<T: Send + Sync + 'static>(
        &mut self,
        mut reserved: ReservedCompanionBranchCell<T>,
        prepared: Arc<T>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.validate_cell(reserved.core())?;
        if reserved.candidate_id != self.binding.candidate_id
            || reserved.current().root_id() != self.binding.expected_root_id
            || reserved.current().commit_id() != self.binding.expected_commit_id
        {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        self.checkpoint()?;
        self.checkpoint()?;
        let pinned_generation = reserved.current().topology_generation();
        let pinned_image = Arc::clone(&reserved.current().image);
        let publication_admission = reserved.publication_admission.take();
        let reservation = reserved.into_reservation();
        let native = PreparedCompanionSlotSwap::prepare(
            reservation,
            pinned_image,
            self.binding.next_root_id,
            self.binding.next_commit_id,
            prepared,
            pinned_generation,
        )?;
        Ok(PreparedPublicationCompanionEffect {
            binding: self.binding.clone(),
            primary: Some(cell_swap::PreparedCellSwap {
                native,
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
        if Arc::ptr_eq(marks.core(), managed.core()) {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        managed.core().retain_source_affinity(self.execution)?;
        let mut effect = self.seal_replacement(marks, next_marks)?;
        let mut source = self.seal_replacement(managed, next_managed)?;
        effect.managed = source.primary.take();
        Ok(effect)
    }
}

#[doc(hidden)]
pub trait PublicationCompanionPreflightOwnerPort<'a>: Sized {
    fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
        source: &'a mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
        execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Result<Self, CompanionPreflightStop>;
}

impl<'a> PublicationCompanionPreflightOwnerPort<'a> for PublicationCompanionPreflight<'a> {
    fn new(
        binding: CandidateCompanionBinding,
        envelope: &'a CanonicalCommitEnvelope,
        control: &'a RelationalOperationControl,
        source: &'a mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
        execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Result<Self, CompanionPreflightStop> {
        Self::new(binding, envelope, control, source, execution)
    }
}
