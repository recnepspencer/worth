//! Register one completely prepared publication after artifact-assembly frames expire.
use super::super::{consult_stage, CommitStage};
use crate::transactions::data::TransactionCommitError;

pub(super) struct CandidateRegistration {
    pub(super) runtime_instance_id: u64,
    pub(super) publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    pub(super) transaction_id: crate::transactions::data::TransactionId,
    pub(super) branch_id: crate::history::data::BranchId,
    pub(super) control: crate::mvcc::RelationalOperationControl,
    pub(super) retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
    pub(super) decision_basis: crate::branch::AdmittedRelationalBranchBasis,
    pub(super) coupled_proposal: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    pub(super) expected_basis: crate::branch::RelationalBranchBasisDescriptor,
    pub(super) expected_root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
    pub(super) publication_cell: crate::branch::RelationalBranchPublicationCell,
}

impl CandidateRegistration {
    #[inline(never)]
    pub(super) fn register(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        mut prepared: super::super::PreparedCommitPublicationExecution,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
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
            control.observe(crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation)
        {
            retention_binding.record_interruption(interruption);
            return Err(TransactionCommitError::interrupted(interruption));
        }
        consult_stage(execution, CommitStage::CandidateRegistration)?;
        let published_snapshot_slot = runtime.reserve_published_snapshot_slot().map_err(|denial| {
            match denial {
                crate::runtime::PublishedSnapshotReservationDenial::Capacity { maximum_handles } => {
                    TransactionCommitError::publication_deferred(
                        crate::mvcc::RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted { maximum_handles },
                    )
                }
                crate::runtime::PublishedSnapshotReservationDenial::CounterOverflow => {
                    TransactionCommitError::publication_failed(crate::mvcc::RelationalPublicationFailure::new(
                        crate::mvcc::RelationalPublicationFailureKind::SnapshotCountOverflow,
                        "published snapshot custody counter overflow before effects",
                    ))
                }
            }
        })?;
        let publication_bases = crate::history::retention::PublicationBasisImage::prepare(
            execution,
            &decision_basis,
            prepared.prepared_root(),
        )?;
        let candidate_retention =
            crate::history::retention::RelationalCandidateRetentionObligation::acquire(
                &retention_binding,
                publication_cell.identity().clone(),
                std::sync::Arc::clone(&expected_root),
                std::sync::Arc::clone(prepared.prepared_root()),
                publication_bases,
            )
            .map_err(|denial| match denial {
                crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
                    TransactionCommitError::publication_deferred(
                        crate::mvcc::RelationalPublicationDeferred::RetentionBackpressure,
                    )
                }
                crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
                    TransactionCommitError::publication_denied(
                        crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                            runtime_instance_id,
                        },
                    )
                }
                crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
                    TransactionCommitError::publication_failed(
                        crate::mvcc::RelationalPublicationFailure::new(
                            crate::mvcc::RelationalPublicationFailureKind::RetentionIdentityExhausted,
                            "prepared candidate retention identity exhausted before effects",
                        ),
                    )
                }
                other @ (crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge
                | crate::history::retention::RelationalRetentionAcquisitionDenial::BindingMismatch) => TransactionCommitError::publication_failed(
                    crate::mvcc::RelationalPublicationFailure::new(
                        crate::mvcc::RelationalPublicationFailureKind::RetentionOwner,
                        format!("prepared candidate retention admission failed: {other:?}"),
                    ),
                ),
            })?;
        if let Some(interruption) =
            control.observe(crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation)
        {
            retention_binding.record_interruption(interruption);
            return Err(TransactionCommitError::interrupted(interruption));
        }
        prepared.release_transaction_retention();
        let mut candidate = crate::mvcc::PreparedRelationalCommitCandidate::new(
            runtime_instance_id,
            runtime.configuration_generation,
            runtime.configuration_epoch.clone(),
            publication_binding,
            transaction_id,
            branch_id,
            expected_basis,
            expected_root,
            publication_cell,
            prepared,
            candidate_retention,
            published_snapshot_slot,
            control,
            runtime
                .config
                .publication
                .policy
                .candidate_max_lifetime_millis,
            runtime.config.publication.policy.max_prepared_candidates,
        )
        .map_err(|stop| match stop {
            crate::mvcc::PreparedRelationalCandidateAdmissionStop::Deferred(deferred) => {
                TransactionCommitError::publication_deferred(deferred)
            }
            crate::mvcc::PreparedRelationalCandidateAdmissionStop::Failed(failure) => {
                TransactionCommitError::publication_failed(failure)
            }
        })?;
        if let Some(proposal) = coupled_proposal {
            candidate.retain_coupled_read(runtime, decision_basis, proposal);
        }
        runtime
            .history
            .record_candidate_preparation(candidate.branch());
        Ok(candidate)
    }
}
