//! Register one completely prepared publication after artifact-assembly frames expire.
use worth_relational_contracts::internal_port as contracts;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::super::{consult_stage, CommitStage};
use worth_relational_mutation::internal_port::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotCapacityOwner;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

pub struct CandidateRegistrationInputs<'a> {
    pub snapshot_capacity: &'a std::sync::Arc<PublishedSnapshotCapacityOwner>,
    pub receiver_epoch: &'a RelationalRuntimeConfigurationSnapshot,
    pub invariant_generation: u64,
    pub maximum_candidates: Option<usize>,
    pub symbols: &'a RuntimeSymbolTable,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub history: &'a RelationalPreparationHistory,
}

pub(super) struct CandidateRegistration {
    pub(super) runtime_instance_id: u64,
    pub(super) publication_binding: crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    pub(super) transaction_id: contracts::transactions::data::primitives::TransactionId,
    pub(super) branch_id: contracts::history::data::coordinates::BranchId,
    pub(super) control: contracts::runtime::operation_control::RelationalOperationControl,
    pub(super) retention_binding: worth_relational_branch::internal_port::history::retention::lane::RelationalBranchRetentionBinding,
    pub(super) decision_basis: worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    pub(super) coupled_proposal:
        Option<worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
    pub(super) expected_basis: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    pub(super) expected_root: std::sync::Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    pub(super) publication_cell: worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
}

impl CandidateRegistration {
    #[inline(never)]
    pub(super) fn register(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: CandidateRegistrationInputs<'_>,
        mut prepared: super::super::PreparedCommitPublicationExecution,
    ) -> Result<crate::mvcc::publication::PreparedRelationalCommitCandidate, TransactionCommitError>
    {
        let Self {
            runtime_instance_id,
            publication_binding,
            transaction_id,
            branch_id,
            control,
            retention_binding,
            decision_basis,
            coupled_proposal,
            expected_basis,
            expected_root,
            publication_cell,
        } = self;
        if let Some(interruption) =
            control.observe(contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation)
        {
            retention_binding.record_interruption(interruption);
            return Err(TransactionCommitError::interrupted(interruption));
        }
        consult_stage(execution, CommitStage::CandidateRegistration)?;
        let published_snapshot_slot = inputs.snapshot_capacity.reserve().map_err(|denial| {
            match denial {
                worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotReservationDenial::Capacity { maximum_handles } => {
                    TransactionCommitError::publication_deferred(
                        contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted { maximum_handles },
                    )
                }
                worth_relational_branch::internal_port::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotReservationDenial::CounterOverflow => {
                    TransactionCommitError::publication_failed(contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                        contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::SnapshotCountOverflow,
                        "published snapshot custody counter overflow before effects",
                    ))
                }
            }
        })?;
        let publication_bases = prepared.take_live_publication_inputs().prepare(
            execution,
            &decision_basis,
            prepared.prepared_root(),
        )?;
        let candidate_retention =
            worth_relational_branch::internal_port::history::retention::obligation::RelationalCandidateRetentionObligation::acquire(
                &retention_binding,
                publication_cell.identity().clone(),
                std::sync::Arc::clone(&expected_root),
                std::sync::Arc::clone(prepared.prepared_root()),
                publication_bases,
            )
            .map_err(|denial| match denial {
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
                    TransactionCommitError::publication_deferred(
                        contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred::RetentionBackpressure,
                    )
                }
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
                    TransactionCommitError::publication_denied(
                        contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial::OwnerUnavailable {
                            runtime_instance_id,
                        },
                    )
                }
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
                    TransactionCommitError::publication_failed(
                        contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                            contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::RetentionIdentityExhausted,
                            "prepared candidate retention identity exhausted before effects",
                        ),
                    )
                }
                other @ (worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::RootSetTooLarge
                | worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::BindingMismatch) => TransactionCommitError::publication_failed(
                    contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                        contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::RetentionOwner,
                        format!("prepared candidate retention admission failed: {other:?}"),
                    ),
                ),
            })?;
        if let Some(interruption) =
            control.observe(contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation)
        {
            retention_binding.record_interruption(interruption);
            return Err(TransactionCommitError::interrupted(interruption));
        }
        prepared.release_transaction_retention();
        let mut candidate = crate::mvcc::publication::PreparedRelationalCommitCandidate::new(
            runtime_instance_id,
            inputs.invariant_generation,
            inputs.receiver_epoch.clone(),
            publication_binding,
            transaction_id,
            branch_id,
            expected_basis,
            expected_root,
            decision_basis.shared_residency(),
            publication_cell,
            prepared,
            candidate_retention,
            published_snapshot_slot,
            control,
            inputs.maximum_candidates,
        )
        .map_err(|stop| match stop {
            crate::mvcc::publication::PreparedRelationalCandidateAdmissionStop::Deferred(
                deferred,
            ) => TransactionCommitError::publication_deferred(deferred),
            crate::mvcc::publication::PreparedRelationalCandidateAdmissionStop::Failed(failure) => {
                TransactionCommitError::publication_failed(failure)
            }
        })?;
        if let Some(proposal) = coupled_proposal {
            candidate.retain_coupled_read(
                inputs.symbols.clone(),
                inputs.instrumentation.clone(),
                decision_basis,
                proposal,
            );
        }
        inputs
            .history
            .record_candidate_preparation(candidate.branch());
        Ok(candidate)
    }
}
