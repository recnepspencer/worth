use std::sync::Arc;

use crate::authority::commit::pipeline::{
    assemble_commit_result, publish_commit_execution, CommitDurableAppendAdmission,
};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::{DeferredPublicationSettlement, DeferredPublicationSettlementError};
use crate::runtime::{
    DeferredRelationalSettlement, PendingRelationalPublicationSettlement,
    PerformedRelationalSettlement, RelationalRuntime, RelationalSettlementClaim,
};
use crate::transactions::data::{CommitResult, TransactionCommitError};

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

/// Why one settlement attempt stopped short of a terminal answer.
pub(crate) enum RelationalSettlementStop {
    /// Movement has not authorized this reservation yet.
    NotYetPerformed,
    /// Durability failed after derived completion; the record is retained.
    DurabilityDeferred {
        carrier: DeferredPublicationSettlement,
        error: TransactionCommitError,
    },
    /// The exact missing durable append failed again.
    DurableAppend(crate::durability::data::DurabilityError),
    /// The retained route no longer matches this runtime's canonical history.
    RouteMissing,
    RouteMismatch,
    /// Derived completion consumed its single-use inputs and then failed.
    Unrecoverable(Option<TransactionCommitError>),
}

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
    /// Immediate settlement, deferred-carrier repair, and commit-identity
    /// repair all enter here, so they share the single per-commit executor gate
    /// and can never repeat a durable or derived effect for the same identity.
    pub(crate) fn execute_pending_settlement(
        &self,
        record: &Arc<PendingRelationalPublicationSettlement>,
        result_owner: RelationalSettlementResultOwner,
    ) -> Result<RelationalSettlementCompletion, RelationalSettlementStop> {
        let execution = record.enter_execution();
        match record.claim(&execution) {
            RelationalSettlementClaim::Immediate(performed) => {
                self.settle_claimed_performed(record, *performed, result_owner)
            }
            RelationalSettlementClaim::DurabilityRepair(deferred) => {
                self.repair_claimed_durability(record, deferred, result_owner)
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
            RelationalSettlementClaim::NotYetPerformed => {
                Err(RelationalSettlementStop::NotYetPerformed)
            }
            RelationalSettlementClaim::Unrecoverable => {
                Err(RelationalSettlementStop::Unrecoverable(None))
            }
        }
    }

    fn settle_claimed_performed(
        &self,
        record: &Arc<PendingRelationalPublicationSettlement>,
        performed: PerformedRelationalSettlement,
        result_owner: RelationalSettlementResultOwner,
    ) -> Result<RelationalSettlementCompletion, RelationalSettlementStop> {
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
        let (mut published, durability_error) =
            match publish_commit_execution(self, completion, published_snapshot_basis) {
                Ok(published) => published,
                Err(error) => {
                    // The single-use publication inputs are gone, so no later
                    // caller can claim work that no longer exists.
                    record.record_unrecoverable();
                    self.publication_binding()
                        .release_pending_settlement(record);
                    return Err(RelationalSettlementStop::Unrecoverable(Some(error)));
                }
            };
        published.append_diagnostics(diagnostic_capture.finish());
        let late_interruption = late_interruption.or_else(|| {
            let event = control.observe(crate::runtime::RelationalInterruptionBoundary::Settlement);
            if let Some(event) = event {
                settlement_retention.record_interruption(event);
            }
            event
        });
        let committed = assemble_commit_result(self, published, late_interruption);
        if let Some(error) = durability_error {
            let snapshot_closeout = self
                .visibility
                .published_snapshot_closeout(committed.snapshot.snapshot_id())
                .expect("deferred publication retains its exact published snapshot");
            record.record_durability_deferred(DeferredRelationalSettlement {
                positioned,
                performed_result: Arc::new(committed),
                snapshot_closeout,
            });
            let carrier = record
                .deferred_carrier()
                .expect("a record just marked durability-deferred exposes its exact carrier");
            return Err(RelationalSettlementStop::DurabilityDeferred { carrier, error });
        }
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
        Ok(completion)
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

    fn repair_claimed_durability(
        &self,
        record: &Arc<PendingRelationalPublicationSettlement>,
        deferred: Box<DeferredRelationalSettlement>,
        result_owner: RelationalSettlementResultOwner,
    ) -> Result<RelationalSettlementCompletion, RelationalSettlementStop> {
        let commit_id = record.commit_id();
        let Some(positioned) = self.history.positioned_canonical_commit(commit_id) else {
            record.restore_deferred(deferred);
            return Err(RelationalSettlementStop::RouteMissing);
        };
        if positioned.as_ref() != deferred.positioned.as_ref() {
            record.restore_deferred(deferred);
            return Err(RelationalSettlementStop::RouteMismatch);
        }
        if self.history.publication_requires_settlement(commit_id) {
            let durable_matches = self
                .durability
                .durable_log_envelope(commit_id)
                .map(|durable| durable.as_ref() == positioned.as_ref());
            match durable_matches {
                Some(true) => {}
                Some(false) => {
                    record.restore_deferred(deferred);
                    return Err(RelationalSettlementStop::RouteMismatch);
                }
                None => {
                    let admission = CommitDurableAppendAdmission::new(
                        self.runtime_instance_id(),
                        commit_id,
                        &positioned.envelope().commit.branch_id,
                    );
                    let authority =
                        crate::durability::authority::DurableAppendAuthority::from_commit(
                            admission,
                        );
                    if let Err(error) = self
                        .durability_authority()
                        .append_commit(authority, positioned.as_ref())
                    {
                        record.restore_deferred(deferred);
                        return Err(RelationalSettlementStop::DurableAppend(error));
                    }
                }
            }
            self.history.mark_publication_settled(commit_id);
        }
        let receipt = positioned.envelope().commit.clone();
        // Durability deferral is reachable from commit-identity repair while a
        // performed witness is still live, so this lane hands the closeout to
        // the same terminal recording rule as immediate settlement.
        let completion = Self::record_terminal_settlement(
            record,
            receipt,
            deferred.performed_result,
            Some(deferred.snapshot_closeout),
            result_owner,
        );
        self.publication_binding()
            .release_pending_settlement(record);
        Ok(completion)
    }

    /// Resolve a settled commit identity that no longer has a pending record.
    ///
    /// Success is the same terminal receipt the executor produced; anything
    /// else is a typed unavailability rather than a silent second effect.
    pub(crate) fn settled_receipt_without_record(
        &self,
        commit_id: CommitId,
    ) -> Result<RelationalCommitReceipt, DeferredPublicationSettlementError> {
        let positioned = self
            .history
            .positioned_canonical_commit(commit_id)
            .ok_or(DeferredPublicationSettlementError::RecoveryUnavailable { commit_id })?;
        if self.history.publication_requires_settlement(commit_id) {
            return Err(DeferredPublicationSettlementError::RecoveryUnavailable { commit_id });
        }
        Ok(positioned.envelope().commit.clone())
    }
}

impl RelationalSettlementStop {
    /// Typed repair posture for a stop that a repair caller observed.
    pub(crate) fn into_repair_error(
        self,
        commit_id: CommitId,
    ) -> DeferredPublicationSettlementError {
        match self {
            Self::NotYetPerformed => {
                DeferredPublicationSettlementError::SettlementInProgress { commit_id }
            }
            Self::DurabilityDeferred { error, .. } => {
                DeferredPublicationSettlementError::DurableAppend(
                    crate::durability::data::DurabilityError::new(
                        crate::durability::data::RecoveryFailureClass::DurableIoFailure,
                        error.detail(),
                    ),
                )
            }
            Self::DurableAppend(error) => DeferredPublicationSettlementError::DurableAppend(error),
            Self::RouteMissing => DeferredPublicationSettlementError::PerformedRouteMissing,
            Self::RouteMismatch => DeferredPublicationSettlementError::PerformedRouteMismatch,
            Self::Unrecoverable(_) => {
                DeferredPublicationSettlementError::RecoveryUnavailable { commit_id }
            }
        }
    }
}
