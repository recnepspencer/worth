use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::history::data::{CommitId, PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::transactions::data::CommitResult;

/// Precomputed settlement input captured before the publication critical
/// section. Holding it here is what lets a lost performed witness stay
/// recoverable: the runtime, not the caller, owns the remaining work.
pub(crate) struct ReservedRelationalSettlement {
    pub(crate) completion: crate::authority::commit::pipeline::PreparedCommitPublicationCompletion,
    pub(crate) published_snapshot_basis:
        crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) control: crate::runtime::RelationalOperationControl,
}

/// The same input after the branch reference moved, authorized against the
/// exact positioned canonical commit that movement produced.
pub(crate) struct PerformedRelationalSettlement {
    pub(crate) completion: crate::authority::commit::pipeline::PreparedCommitPublicationCompletion,
    pub(crate) published_snapshot_basis:
        crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) control: crate::runtime::RelationalOperationControl,
    pub(crate) positioned: Arc<PositionedCanonicalCommit>,
    pub(crate) settlement_retention:
        crate::history::retention::RelationalPerformedSettlementObligation,
    pub(crate) late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
}

/// Settlement whose derived completion already ran and whose only missing step
/// is the exact durable append.
///
/// `snapshot_closeout` is the runtime's sole obligation for this deferral. The
/// external carrier this record exposes is a view of the record, so it may not
/// hold a second copy of the obligation.
pub(crate) struct DeferredRelationalSettlement {
    pub(crate) positioned: Arc<PositionedCanonicalCommit>,
    pub(crate) performed_result: Arc<CommitResult>,
    pub(crate) snapshot_closeout: crate::runtime::PublishedSnapshotCloseout,
}

/// The one terminal answer for a settled commit identity. Every later caller
/// reads this instead of repeating a durable or derived effect.
///
/// `result` is retained only when the executor was not the natural owner of the
/// commit result, which is exactly the repair case. Immediate settlement hands
/// its result to the witness holder instead of copying it.
///
/// `closeout` travels with `result` and moves to the same claim: whoever takes
/// the commit result also takes releasing its published snapshot. If no caller
/// ever claims it, dropping this record drops the closeout, which closes the
/// handle rather than leaking it.
pub(crate) struct SettledRelationalSettlement {
    pub(crate) receipt: RelationalCommitReceipt,
    pub(crate) result: Option<Arc<CommitResult>>,
    pub(crate) closeout: Option<crate::runtime::PublishedSnapshotCloseout>,
}

enum PendingRelationalSettlementState {
    Reserved(Box<ReservedRelationalSettlement>),
    Performed(Box<PerformedRelationalSettlement>),
    DurabilityDeferred(Box<DeferredRelationalSettlement>),
    Executing,
    Settled(Box<SettledRelationalSettlement>),
    /// The derived completion consumed its single-use inputs and then failed.
    /// The route cannot be replayed, so repair reports this as a typed
    /// unavailability rather than pretending work remains claimable.
    Unrecoverable,
}

/// One runtime-owned pending publication settlement, keyed by the candidate's
/// owner-issued commit identity.
///
/// `execution_gate` is the single-executor gate shared by immediate settlement,
/// deferred-carrier repair, and commit-identity repair. It is per-commit and is
/// deliberately not the registry index lock, which never spans an effect.
pub(crate) struct PendingRelationalPublicationSettlement {
    commit_id: CommitId,
    runtime_instance_id: u64,
    state: Mutex<PendingRelationalSettlementState>,
    execution_gate: Mutex<()>,
    abandoned_capabilities: AtomicU64,
}

/// What the single executor is authorized to do for one claim.
pub(crate) enum RelationalSettlementClaim {
    Immediate(Box<PerformedRelationalSettlement>),
    DurabilityRepair(Box<DeferredRelationalSettlement>),
    AlreadySettled(Box<SettledRelationalSettlement>),
    NotYetPerformed,
    Unrecoverable,
}

/// Proof that its holder is the one executor for this exact record.
pub(crate) struct RelationalSettlementExecution<'record> {
    _gate: MutexGuard<'record, ()>,
}

impl PendingRelationalPublicationSettlement {
    pub(crate) fn reserved(
        commit_id: CommitId,
        runtime_instance_id: u64,
        reserved: ReservedRelationalSettlement,
    ) -> Self {
        Self {
            commit_id,
            runtime_instance_id,
            state: Mutex::new(PendingRelationalSettlementState::Reserved(Box::new(
                reserved,
            ))),
            execution_gate: Mutex::new(()),
            abandoned_capabilities: AtomicU64::new(0),
        }
    }

    pub(crate) const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub(crate) const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    fn state(&self) -> MutexGuard<'_, PendingRelationalSettlementState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enter the single-executor gate. Concurrent immediate settlement and
    /// repair converge here instead of repeating durability or derived work.
    pub(crate) fn enter_execution(&self) -> RelationalSettlementExecution<'_> {
        RelationalSettlementExecution {
            _gate: self
                .execution_gate
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    /// Authorize the already-installed reservation against the exact positioned
    /// canonical commit produced by movement. Returns false when the record was
    /// already authorized, which cannot happen for one linearized candidate.
    pub(crate) fn authorize_performed(
        &self,
        positioned: Arc<PositionedCanonicalCommit>,
        settlement_retention: crate::history::retention::RelationalPerformedSettlementObligation,
        late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
    ) -> bool {
        let mut state = self.state();
        if !matches!(&*state, PendingRelationalSettlementState::Reserved(_)) {
            return false;
        }
        let PendingRelationalSettlementState::Reserved(reserved) =
            std::mem::replace(&mut *state, PendingRelationalSettlementState::Executing)
        else {
            unreachable!("reserved settlement was observed immediately above");
        };
        *state =
            PendingRelationalSettlementState::Performed(Box::new(PerformedRelationalSettlement {
                completion: reserved.completion,
                published_snapshot_basis: reserved.published_snapshot_basis,
                control: reserved.control,
                positioned,
                settlement_retention,
                late_interruption,
            }));
        true
    }

    /// Take the outstanding work. The caller proves single execution by holding
    /// the gate, so no second executor can observe the same work.
    pub(crate) fn claim(
        &self,
        _execution: &RelationalSettlementExecution<'_>,
    ) -> RelationalSettlementClaim {
        let mut state = self.state();
        match std::mem::replace(&mut *state, PendingRelationalSettlementState::Executing) {
            PendingRelationalSettlementState::Reserved(reserved) => {
                *state = PendingRelationalSettlementState::Reserved(reserved);
                RelationalSettlementClaim::NotYetPerformed
            }
            PendingRelationalSettlementState::Performed(performed) => {
                RelationalSettlementClaim::Immediate(performed)
            }
            PendingRelationalSettlementState::DurabilityDeferred(deferred) => {
                RelationalSettlementClaim::DurabilityRepair(deferred)
            }
            PendingRelationalSettlementState::Settled(mut settled) => {
                // The terminal answer stays; the one commit result and its
                // published-snapshot closeout transfer to this single claimant.
                let claimed = SettledRelationalSettlement {
                    receipt: settled.receipt.clone(),
                    result: settled.result.take(),
                    closeout: settled.closeout.take(),
                };
                *state = PendingRelationalSettlementState::Settled(settled);
                RelationalSettlementClaim::AlreadySettled(Box::new(claimed))
            }
            PendingRelationalSettlementState::Unrecoverable => {
                *state = PendingRelationalSettlementState::Unrecoverable;
                RelationalSettlementClaim::Unrecoverable
            }
            PendingRelationalSettlementState::Executing => {
                unreachable!("the executor gate excludes a second in-flight claim")
            }
        }
    }

    /// Terminal settlement. Recording the one answer here is what lets a
    /// repeated or concurrent caller return it without another effect.
    pub(crate) fn record_settled(
        &self,
        receipt: RelationalCommitReceipt,
        result: Option<Arc<CommitResult>>,
        closeout: Option<crate::runtime::PublishedSnapshotCloseout>,
    ) {
        *self.state() =
            PendingRelationalSettlementState::Settled(Box::new(SettledRelationalSettlement {
                receipt,
                result,
                closeout,
            }));
    }

    /// The derived completion failed after consuming its single-use inputs.
    pub(crate) fn record_unrecoverable(&self) {
        *self.state() = PendingRelationalSettlementState::Unrecoverable;
    }

    /// Durability failed after the derived completion ran. The record is
    /// retained so repair stays addressable by commit identity alone.
    pub(crate) fn record_durability_deferred(&self, deferred: DeferredRelationalSettlement) {
        *self.state() = PendingRelationalSettlementState::DurabilityDeferred(Box::new(deferred));
    }

    /// Return unclaimed work when the executor stopped before any effect.
    pub(crate) fn restore_deferred(&self, deferred: Box<DeferredRelationalSettlement>) {
        *self.state() = PendingRelationalSettlementState::DurabilityDeferred(deferred);
    }

    /// Dropping `PerformedRelationalCommit` records abandonment only. It may
    /// not release the obligation, mark the route settled, remove this record,
    /// or make repair unavailable.
    pub(crate) fn record_capability_abandonment(&self) {
        self.abandoned_capabilities.fetch_add(1, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(crate) fn abandoned_capability_count(&self) -> u64 {
        self.abandoned_capabilities.load(Ordering::Relaxed)
    }

    /// The accepted external carrier for a durability-deferred settlement.
    /// It is a view of this record, never a second registry.
    pub(crate) fn deferred_carrier(
        &self,
    ) -> Option<crate::publication::data::DeferredPublicationSettlement> {
        match &*self.state() {
            PendingRelationalSettlementState::DurabilityDeferred(deferred) => Some(
                crate::publication::data::DeferredPublicationSettlement::new(
                    self.runtime_instance_id,
                    Arc::clone(&deferred.positioned),
                    deferred.performed_result.as_ref().clone(),
                ),
            ),
            _ => None,
        }
    }

    /// Exact route retained by a durability-deferred record, so a presented
    /// carrier can be checked against it before any repair effect.
    pub(crate) fn deferred_route(&self) -> Option<Arc<PositionedCanonicalCommit>> {
        match &*self.state() {
            PendingRelationalSettlementState::DurabilityDeferred(deferred) => {
                Some(Arc::clone(&deferred.positioned))
            }
            _ => None,
        }
    }
}

impl std::fmt::Debug for PendingRelationalPublicationSettlement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingRelationalPublicationSettlement")
            .field("commit_id", &self.commit_id)
            .field("runtime_instance_id", &self.runtime_instance_id)
            .finish_non_exhaustive()
    }
}
