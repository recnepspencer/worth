use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;
pub mod caller_association;
pub mod changed_records;
pub mod coupled_read;
pub mod current_read;
pub mod index_omission;
pub mod managed_source;
pub mod output_adoption;
mod payload_backing;
pub mod publication_parts;
pub(crate) use payload_backing::{SharedCandidatePayload, WeakCandidatePayload};
pub mod read_condition;
pub mod required_outputs;
pub use output_adoption::RelationalCurrentOutputAdoption;
pub mod retained_selection;
use std::cell::Cell;
use std::marker::PhantomData;

use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use worth_relational_branch::internal_port::branch::home_program_step::AdmittedRelationalHomeProgramStep;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_durability::internal_port::durability::authority::recovery::codec_transition::publication_custody::RecoveredCodecPublicationCustody;
use contracts::history::data::coordinates::BranchId;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedRelationalIndexDefinitions;
use contracts::transactions::data::primitives::TransactionId;
use std::sync::Arc;

/// Opaque, single-use owner artifact whose fallible commit preparation is complete.
///
/// A candidate has no publication method. Only the Relational runtime owner can
/// consume it, either through publication or explicit discard.
pub struct PreparedRelationalCommitCandidate {
    runtime_instance_id: u64,
    configuration_invariant_generation: u64,
    receiver_epoch: mutation::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot,
    publication_binding: crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    transaction_id: TransactionId,
    branch_id: BranchId,
    candidate_id: u64,
    _payload: SharedCandidatePayload,
    reservation_count: usize,
    consumed: bool,
    _not_sync: PhantomData<Cell<()>>,
}

pub struct CandidatePayload {
    coupled_read: Option<coupled_read::CoupledCandidateReadContext>,
    current_read: Option<AdmittedRelationalBranchBasis>,
    managed_source_reads: Option<super::companion::ManagedSourceReadConditions>,
    companion_read: Option<super::companion::CompanionReadCondition>,
    expected_basis: RelationalBranchBasisDescriptor,
    expected_root: Arc<RelationalBranchRoot>,
    publication_cell: worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
    execution: crate::authority::commit::pipeline::PreparedCommitPublicationExecution,
    retention: worth_relational_branch::internal_port::history::retention::obligation::RelationalCandidateRetentionObligation,
    published_snapshot_slot:
        worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    control: contracts::runtime::operation_control::RelationalOperationControl,
}

pub struct PreparedRelationalPublicationParts {
    pub current_read: Option<AdmittedRelationalBranchBasis>,
    pub managed_source_reads: Option<super::companion::ManagedSourceReadConditions>,
    pub companion_read: Option<super::companion::CompanionReadCondition>,
    pub runtime_instance_id: u64,
    pub candidate_id: u64,
    pub publication_binding: crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    pub expected: RelationalBranchBasisDescriptor,
    pub expected_root: Arc<RelationalBranchRoot>,
    pub publication_cell: worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
    pub movement: super::PreparedCanonicalBranchMovement,
    pub completion: crate::authority::commit::pipeline::PreparedCommitPublicationCompletion,
    pub candidate_retention:
        worth_relational_branch::internal_port::history::retention::obligation::RelationalCandidateRetentionObligation,
    pub control: contracts::runtime::operation_control::RelationalOperationControl,
}

pub enum PreparedRelationalCandidateAdmissionStop {
    Deferred(contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred),
    Failed(contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure),
}

impl PreparedRelationalCommitCandidate {
    pub(crate) fn new(
        runtime_instance_id: u64,
        configuration_invariant_generation: u64,
        receiver_epoch: mutation::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot,
        publication_binding: crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
        transaction_id: TransactionId,
        branch_id: BranchId,
        expected_basis: RelationalBranchBasisDescriptor,
        expected_root: Arc<RelationalBranchRoot>,
        publication_cell: worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
        execution: crate::authority::commit::pipeline::PreparedCommitPublicationExecution,
        retention: worth_relational_branch::internal_port::history::retention::obligation::RelationalCandidateRetentionObligation,
        published_snapshot_slot: worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
        control: contracts::runtime::operation_control::RelationalOperationControl,
        maximum_candidates: Option<usize>,
    ) -> Result<Self, PreparedRelationalCandidateAdmissionStop> {
        let reservation_count = execution.reservation_count();
        let payload = SharedCandidatePayload::new(CandidatePayload {
            coupled_read: None,
            current_read: None,
            companion_read: None,
            managed_source_reads: None,
            expected_basis,
            expected_root,
            publication_cell,
            execution,
            retention,
            published_snapshot_slot,
            control,
        });
        let candidate_id = publication_binding
            .register_candidate(maximum_candidates, &payload)
            .map_err(|denial| match denial {
                crate::runtime::state::runtime_state::publication_lifecycle::RelationalCandidateRegistrationDenial::CapacityExhausted {
                    maximum_candidates,
                } => PreparedRelationalCandidateAdmissionStop::Deferred(
                    contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred::CandidateCapacityExhausted {
                        maximum_candidates,
                    },
                ),
                crate::runtime::state::runtime_state::publication_lifecycle::RelationalCandidateRegistrationDenial::IdentityExhausted => {
                    PreparedRelationalCandidateAdmissionStop::Failed(
                        contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                            contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::CandidateIdentityExhausted,
                            "prepared candidate identity space is exhausted",
                        ),
                    )
                }
            })?;
        Ok(Self {
            runtime_instance_id,
            configuration_invariant_generation,
            receiver_epoch,
            publication_binding,
            transaction_id,
            branch_id,
            candidate_id,
            _payload: payload,
            reservation_count,
            consumed: false,
            _not_sync: PhantomData,
        })
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn branch(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub(crate) fn belongs_to_receiver_epoch(
        &self,
        epoch: &mutation::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot,
    ) -> bool {
        self.receiver_epoch.same_epoch(epoch)
    }

    pub(crate) fn configuration_invariant_generation(&self) -> u64 {
        self.configuration_invariant_generation
    }

    pub(crate) fn expected_basis(&self) -> Option<RelationalBranchBasisDescriptor> {
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|payload| payload.expected_basis.clone())
    }

    /// Basis, successor parent count, and fixed-size root/commit identities.
    pub(crate) fn with_successor_shape<R>(
        &self,
        read: impl FnOnce(
            &RelationalBranchBasisDescriptor,
            usize,
            u64,
            contracts::history::data::coordinates::CommitId,
        ) -> R,
    ) -> Option<R> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let payload = payload.as_ref()?;
        Some(read(
            &payload.expected_basis,
            payload.execution.successor_parent_count(),
            payload.execution.prepared_root().id(),
            payload.execution.canonical_envelope().commit.commit_id,
        ))
    }

    pub(crate) fn belongs_to_publication_owner(
        &self,
        binding: &crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    ) -> bool {
        self.publication_binding.belongs_to_same_owner(binding)
    }

    pub(crate) fn reservation_count(&self) -> usize {
        self.reservation_count
    }

    pub(crate) fn index_preparation_roots(
        &self,
    ) -> Option<(Arc<RelationalBranchRoot>, Arc<RelationalBranchRoot>)> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let payload = payload.as_ref()?;
        Some((
            Arc::clone(&payload.expected_root),
            Arc::clone(payload.execution.prepared_root()),
        ))
    }

    pub(crate) fn change_summary_basis_and_roots(
        &self,
    ) -> Option<(
        RelationalBranchBasisDescriptor,
        Arc<RelationalBranchRoot>,
        Arc<RelationalBranchRoot>,
    )> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let payload = payload.as_ref()?;
        Some((
            payload.expected_basis.clone(),
            Arc::clone(&payload.expected_root),
            Arc::clone(payload.execution.prepared_root()),
        ))
    }

    pub(crate) fn recovered_codec_publication_custody(
        &self,
    ) -> Option<Arc<RecoveredCodecPublicationCustody>> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| {
                payload
                    .execution
                    .recovered_codec_publication_custody()
                    .cloned()
            })
    }

    pub(crate) fn home_step(&self) -> Option<Arc<AdmittedRelationalHomeProgramStep>> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| payload.execution.home_step().cloned())
    }

    pub(crate) fn home_index_definitions(&self) -> Option<Arc<PreparedRelationalIndexDefinitions>> {
        self._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|payload| payload.execution.home_index_definitions())
    }

    pub(crate) fn covers_home_definitions(
        &self,
        definitions: &PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        match self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            Some(payload) => payload
                .execution
                .covers_home_definitions(definitions, execution),
            None => Ok(false),
        }
    }

    pub(crate) fn indexes_already_prepared(&self) -> bool {
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .is_some_and(|payload| payload.execution.has_prepared_indexes())
    }

    pub(crate) fn attach_prepared_indexes(
        &mut self,
        prepared: worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication,
    ) -> Result<(), contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind> {
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let payload = payload
            .as_mut()
            .ok_or(contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind::CandidateUnavailable)?;
        payload.execution.attach_prepared_indexes(prepared)
    }
}

impl Drop for PreparedRelationalCommitCandidate {
    fn drop(&mut self) {
        if !self.consumed {
            self.publication_binding
                .discard_candidate(self.candidate_id);
        }
    }
}

impl std::fmt::Debug for PreparedRelationalCommitCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedRelationalCommitCandidate")
            .field("transaction_id", &self.transaction_id)
            .field("branch_id", &self.branch_id)
            .field("reservation_count", &self.reservation_count())
            .finish_non_exhaustive()
    }
}

/// Evidence returned when a prepared candidate is consumed without publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedRelationalCommitCandidate {
    transaction_id: TransactionId,
    branch_id: BranchId,
    released_record_reservation_count: usize,
}

impl DiscardedRelationalCommitCandidate {
    pub(crate) fn from_candidate(candidate: &PreparedRelationalCommitCandidate) -> Self {
        Self {
            transaction_id: candidate.transaction_id,
            branch_id: candidate.branch_id.clone(),
            released_record_reservation_count: candidate.reservation_count(),
        }
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn branch(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn released_record_reservation_count(&self) -> usize {
        self.released_record_reservation_count
    }
}

#[path = "candidate_owner_port.rs"]
mod owner_port;
pub use owner_port::{
    DiscardedRelationalCommitCandidateOwnerPort, PreparedRelationalCommitCandidateOwnerPort,
};

mod fixture_port;
pub use fixture_port::PreparedRelationalCommitCandidateFixturePort;
