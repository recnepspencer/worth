//! Independently borrowable Relational owner settlement service.

use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;

use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::state::runtime_state::WeakRelationalRuntimeState;
use crate::runtime::{RelationalRuntime, RelationalRuntimeOwnerBinding};
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

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
    state: WeakRelationalRuntimeState,
}

impl RelationalSettlementPort {
    pub(crate) fn new(
        runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        publication_binding: RelationalRuntimePublicationBinding,
        state: WeakRelationalRuntimeState,
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
        performed: crate::mvcc::publication::outcome::PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> {
        let Some(owner) = self.owner() else {
            return Err(TransactionCommitError::publication_denied(
                crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::OwnerUnavailable {
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
