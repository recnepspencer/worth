//! Independently borrowable Relational owner settlement service.

use std::sync::Weak;

use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::{
    RelationalRuntime, RelationalRuntimeOwnerBinding, RelationalRuntimePublicationBinding,
    RelationalRuntimeState,
};
use crate::transactions::data::{CommitResult, TransactionCommitError};

/// Cloneable settlement and repair service bound to one live relational runtime
/// owner.
///
/// The service addresses its owner's exact state by weak binding, so it never
/// keeps a finished runtime alive and answers a typed owner-unavailable posture
/// once the owner is gone. It never reconstructs settlement authority: the
/// commit identity it accepts is only a lookup key into the runtime's own
/// pending-settlement registry.
#[derive(Debug, Clone)]
pub struct RelationalSettlementPort {
    runtime_instance_id: u64,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication_binding: RelationalRuntimePublicationBinding,
    state: Weak<RelationalRuntimeState>,
}

impl RelationalSettlementPort {
    pub(crate) fn new(
        runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        publication_binding: RelationalRuntimePublicationBinding,
        state: Weak<RelationalRuntimeState>,
    ) -> Self {
        Self {
            runtime_instance_id,
            lifecycle,
            publication_binding,
            state,
        }
    }

    /// The runtime instance this service settles for.
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    /// Complete derived publication for a movement already performed, and
    /// made durable, through the publication service.
    ///
    /// The work was installed in the runtime's pending-settlement registry
    /// before that movement, so this is the witness holder taking its turn at
    /// the one executor gate.
    pub fn settle_performed_publication(
        &self,
        performed: crate::mvcc::PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> {
        let Some(owner) = self.owner() else {
            return Err(TransactionCommitError::publication_denied(
                crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                    runtime_instance_id: self.runtime_instance_id,
                },
            ));
        };
        owner.settle_performed_publication(performed)
    }

    /// Settle a performed publication this runtime retains, addressed by
    /// commit identity, when the caller that held its performed witness was
    /// lost.
    pub fn repair_pending_publication_settlement(
        &self,
        commit_id: CommitId,
    ) -> Result<RelationalCommitReceipt, PublicationSettlementRepairError> {
        let Some(owner) = self.owner() else {
            return Err(self.owner_unavailable());
        };
        owner.repair_pending_publication_settlement(commit_id)
    }

    /// Whether this runtime still retains an unsettled record for the identity.
    pub fn retains_pending_settlement(&self, commit_id: CommitId) -> bool {
        self.publication_binding
            .pending_settlement(commit_id)
            .is_some()
    }

    /// A handle for one admitted settlement operation.
    ///
    /// The lifecycle gate is consulted first, so a runtime that has stopped
    /// accepting work denies before any state is reached. The handle then
    /// carries that exact admission for the whole operation, which is what
    /// keeps the owner from finishing its close while work is outstanding, and
    /// keeps the state alive for exactly that long. It carries no close
    /// authority, so completing here can never close the runtime it borrowed.
    fn owner(&self) -> Option<RelationalRuntime> {
        let operation = self.lifecycle.admit()?;
        Some(RelationalRuntime::admitted(
            self.state.upgrade()?,
            operation,
        ))
    }

    fn owner_unavailable(&self) -> PublicationSettlementRepairError {
        PublicationSettlementRepairError::OwnerUnavailable {
            runtime_instance_id: self.runtime_instance_id,
        }
    }
}

impl RelationalRuntime {
    /// The independently borrowable settlement service for this runtime.
    ///
    /// Settlement is the obligation a performed publication transfers to its
    /// caller. The service owns
    /// [`RelationalSettlementPort::settle_performed_publication`] for the
    /// ordinary path and
    /// [`RelationalSettlementPort::repair_pending_publication_settlement`] for a
    /// performed witness that was lost. Dropping performed evidence without settling it is
    /// not success.
    ///
    /// Like
    /// [`RelationalRuntime::preparation_port`](crate::runtime::RelationalRuntime::preparation_port),
    /// this service is obtained from a shared borrow and is
    /// `Clone + Send + Sync`. The runnable owner workflow is
    /// `examples/branch_local_mvcc.rs`.
    pub fn settlement_port(&self) -> RelationalSettlementPort {
        RelationalSettlementPort::new(
            self.runtime_instance_id(),
            self.owner_binding(),
            self.publication_binding(),
            self.state_binding(),
        )
    }
}
