use std::sync::Arc;

use crate::authority::commit::pipeline::{assemble_commit_result, publish_commit_execution};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::{
    PendingRelationalPublicationSettlement, PerformedRelationalSettlement, RelationalRuntime,
    RelationalSettlementClaim,
};
use crate::transactions::data::CommitResult;

/// Terminal answer produced by the one executor of a pending settlement.
pub(crate) enum RelationalSettlementCompletion {
    /// This caller ran the terminal effect and owns its exact commit result.
    Performed {
        receipt: RelationalCommitReceipt,
        committed: CommitResult,
    },
    /// Another executor already produced the terminal answer for this identity.
    Repeated {
        receipt: RelationalCommitReceipt,
        committed: Option<Arc<CommitResult>>,
    },
}

/// Movement has not authorized this reservation yet, so there is no performed
/// work to settle. Settlement of performed work cannot stop: the commit is
/// already durable, and derived installation is infallible.
#[derive(Debug)]
pub(crate) struct RelationalSettlementNotYetPerformed;

/// Who owns the commit result this settlement produces.
///
/// The published snapshot a settlement opens is released by whoever holds the
/// commit result, so exactly one of these two answers applies per execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalSettlementResultOwner {
    /// The caller surrendered its performed witness and receives the result.
    Witness,
    /// The caller asked only for a receipt, so the runtime retains the result
    /// for a still-live witness and closes its snapshot if none claims it.
    Runtime,
}

impl RelationalSettlementCompletion {
    pub(crate) fn receipt(&self) -> &RelationalCommitReceipt {
        match self {
            Self::Performed { receipt, .. } | Self::Repeated { receipt, .. } => receipt,
        }
    }
}

impl RelationalRuntime {
    /// Run the one outstanding settlement for a pending record.
    ///
    /// Immediate settlement and commit-identity repair both enter here, so they
    /// share the single per-commit executor gate and can never repeat a derived
    /// effect for the same identity.
    pub(crate) fn execute_pending_settlement(
        &self,
        record: &Arc<PendingRelationalPublicationSettlement>,
        result_owner: RelationalSettlementResultOwner,
    ) -> Result<RelationalSettlementCompletion, RelationalSettlementNotYetPerformed> {
        let execution = record.enter_execution();
        match record.claim(&execution) {
            RelationalSettlementClaim::Immediate(performed) => {
                Ok(self.settle_claimed_performed(record, *performed, result_owner))
            }
            RelationalSettlementClaim::AlreadySettled(settled) => {
                let receipt = settled.receipt.clone();
                match result_owner {
                    // Taking the retained result also takes responsibility for
                    // releasing its published snapshot.
                    RelationalSettlementResultOwner::Witness => {
                        if let Some(closeout) = settled.closeout {
                            closeout.transfer_release_obligation();
                        }
                        Ok(RelationalSettlementCompletion::Repeated {
                            receipt,
                            committed: settled.result,
                        })
                    }
                    // A receipt-only caller leaves the result where a still-live
                    // witness can still claim it.
                    RelationalSettlementResultOwner::Runtime => {
                        record.record_settled(settled.receipt, settled.result, settled.closeout);
                        Ok(RelationalSettlementCompletion::Repeated {
                            receipt,
                            committed: None,
                        })
                    }
                }
            }
            RelationalSettlementClaim::NotYetPerformed => Err(RelationalSettlementNotYetPerformed),
        }
    }

    fn settle_claimed_performed(
        &self,
        record: &Arc<PendingRelationalPublicationSettlement>,
        performed: PerformedRelationalSettlement,
        result_owner: RelationalSettlementResultOwner,
    ) -> RelationalSettlementCompletion {
        let PerformedRelationalSettlement {
            completion,
            published_snapshot_basis,
            control,
            positioned,
            settlement_retention,
            late_interruption,
        } = performed;
        let commit_id = positioned.envelope().commit.commit_id;
        let diagnostic_capture = self.publication.diagnostics.begin_operation_capture();
        let mut published = publish_commit_execution(self, completion, published_snapshot_basis);
        published.append_diagnostics(diagnostic_capture.finish());
        let late_interruption = late_interruption.or_else(|| {
            let event = control.observe(crate::runtime::RelationalInterruptionBoundary::Settlement);
            if let Some(event) = event {
                settlement_retention.record_interruption(event);
            }
            event
        });
        let committed = assemble_commit_result(self, published, late_interruption);
        self.history.mark_publication_settled(commit_id);
        let receipt = positioned.envelope().commit.clone();
        // A receipt-only executor keeps the published snapshot open, because a
        // performed witness may still be live and owed this exact result.
        let closeout = match result_owner {
            RelationalSettlementResultOwner::Witness => None,
            RelationalSettlementResultOwner::Runtime => self
                .visibility
                .published_snapshot_closeout(committed.snapshot.snapshot_id()),
        };
        let completion = Self::record_terminal_settlement(
            record,
            receipt,
            Arc::new(committed),
            closeout,
            result_owner,
        );
        self.publication_binding()
            .release_pending_settlement(record);
        completion
    }

    /// Record the one terminal answer and settle who releases the published
    /// snapshot the commit result names.
    ///
    /// Both settlement lanes reach here, and a performed witness may still be
    /// live on either of them, so the closeout is never closed here. A witness
    /// claim moves the release obligation out with the result; a retention no
    /// caller ever claims closes when this record is dropped, which is the one
    /// remaining release because no other party may hold this obligation.
    fn record_terminal_settlement(
        record: &Arc<PendingRelationalPublicationSettlement>,
        receipt: RelationalCommitReceipt,
        committed: Arc<CommitResult>,
        closeout: Option<crate::runtime::PublishedSnapshotCloseout>,
        result_owner: RelationalSettlementResultOwner,
    ) -> RelationalSettlementCompletion {
        match result_owner {
            RelationalSettlementResultOwner::Witness => {
                if let Some(closeout) = closeout {
                    closeout.transfer_release_obligation();
                }
                record.record_settled(receipt.clone(), None, None);
                RelationalSettlementCompletion::Performed {
                    receipt,
                    committed: Arc::try_unwrap(committed)
                        .unwrap_or_else(|shared| shared.as_ref().clone()),
                }
            }
            RelationalSettlementResultOwner::Runtime => {
                record.record_settled(receipt.clone(), Some(committed), closeout);
                RelationalSettlementCompletion::Repeated {
                    receipt,
                    committed: None,
                }
            }
        }
    }

    /// Resolve a settled commit identity that no longer has a pending record.
    ///
    /// Success is the same terminal receipt the executor produced; anything
    /// else is a typed unavailability rather than a silent second effect.
    pub(crate) fn settled_receipt_without_record(
        &self,
        commit_id: CommitId,
    ) -> Result<RelationalCommitReceipt, PublicationSettlementRepairError> {
        let positioned = self
            .history
            .positioned_canonical_commit(commit_id)
            .ok_or(PublicationSettlementRepairError::RecoveryUnavailable { commit_id })?;
        if self.history.publication_requires_settlement(commit_id) {
            return Err(PublicationSettlementRepairError::RecoveryUnavailable { commit_id });
        }
        Ok(positioned.envelope().commit.clone())
    }
}
