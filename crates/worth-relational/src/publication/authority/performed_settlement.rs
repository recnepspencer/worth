use std::sync::Arc;

use super::PublicationAuthority;
use crate::authority::commit::pipeline::PublishedCommitExecution;
use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use crate::runtime::state::subsystems::history::history_canonical_routes::RelationalPublicationHistory;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

use super::pending_settlement::{
    RelationalSettlementCompletion, RelationalSettlementResultOwner, SettlementPublication,
};

/// Exact registry and restart receipt custody from one installed owner.
pub(crate) struct RelationalSettlementBinding<'runtime> {
    pub(crate) runtime_instance_id: u64,
    pub(crate) publication: RelationalRuntimePublicationBinding,
    pub(crate) history: &'runtime RelationalPublicationHistory,
    pub(crate) checkpoint: &'runtime RelationalCheckpointManifestSelection,
}

impl PublicationAuthority<'_> {
    pub(crate) fn settle_performed_publication(
        &self,
        binding: &RelationalSettlementBinding<'_>,
        performed: crate::mvcc::publication::outcome::PerformedRelationalCommit,
        publish: impl FnOnce(SettlementPublication) -> PublishedCommitExecution,
    ) -> Result<CommitResult, TransactionCommitError> {
        let (positioned, record) = performed.into_settlement_parts();
        debug_assert_eq!(
            /* Proof: PerformedRelationalCommit::record is called only after authorizing the exact positioned commit in its pending record. The non-Clone witness keeps that record and cannot change either commit identity. */
            positioned.envelope().commit.commit_id,
            record.commit_id()
        );
        if record.runtime_instance_id() != binding.runtime_instance_id {
            return Err(TransactionCommitError::publication_denied(
                crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::ForeignRuntime {
                    expected_runtime_instance_id: binding.runtime_instance_id,
                    actual_runtime_instance_id: record.runtime_instance_id(),
                },
            ));
        }
        let completion = self
            .execute_pending_settlement(
                &record, RelationalSettlementResultOwner::Witness, binding.history, &binding.publication, publish,
            )
            .expect(/* Proof: PerformedRelationalCommit is non-Clone and record is called only after authorize transitions this exact reservation to Performed. claim runs under enter_execution; receipt-only Runtime repair uses record_settled to retain the result, and only this consuming Witness call may take it. The witness pins the record even after registry acknowledgement. */ "a performed witness exists only after its reservation is authorized");
        match completion {
            RelationalSettlementCompletion::Performed { committed, .. } => Ok(committed),
            RelationalSettlementCompletion::Repeated { committed, .. } => {
                let retained = committed.expect(/* Proof: PerformedRelationalCommit is non-Clone and record is called only after authorize transitions this exact reservation to Performed. claim runs under enter_execution; receipt-only Runtime repair uses record_settled to retain the result, and only this consuming Witness call may take it. The witness pins the record even after registry acknowledgement. */
                    "a receipt-only executor retains the commit result for its live witness",
                );
                Ok(Arc::try_unwrap(retained).unwrap_or_else(|shared| shared.as_ref().clone()))
            }
        }
    }

    pub(crate) fn repair_pending_publication_settlement(
        &self,
        binding: &RelationalSettlementBinding<'_>,
        commit_id: crate::history::data::CommitId,
        publish: impl FnOnce(SettlementPublication) -> PublishedCommitExecution,
    ) -> Result<crate::history::data::RelationalCommitReceipt, PublicationSettlementRepairError>
    {
        let Some(record) = binding.publication.pending_settlement(commit_id) else {
            if binding.publication.settlement_admission_is_closed() {
                return Err(PublicationSettlementRepairError::OwnerUnavailable {
                    runtime_instance_id: binding.runtime_instance_id,
                });
            }
            return super::settled_receipt::settled_receipt_without_record(
                binding.history,
                binding.checkpoint,
                commit_id,
            );
        };
        if record.runtime_instance_id() != binding.runtime_instance_id {
            return Err(PublicationSettlementRepairError::ForeignRuntime {
                expected_runtime_instance_id: binding.runtime_instance_id,
                actual_runtime_instance_id: record.runtime_instance_id(),
            });
        }
        self.execute_pending_settlement(
            &record,
            RelationalSettlementResultOwner::Runtime,
            binding.history,
            &binding.publication,
            publish,
        )
        .map(|completion| completion.receipt().clone())
        .map_err(|_| PublicationSettlementRepairError::SettlementInProgress { commit_id })
    }
}
