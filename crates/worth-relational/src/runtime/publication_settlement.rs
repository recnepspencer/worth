//! Bind the installed runtime to the effect authorized by Publication.
use super::RelationalRuntime;
use crate::authority::commit::pipeline::{publish_commit_execution, PublishedCommitExecution};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::mvcc::publication::outcome::PerformedRelationalCommit;
use crate::publication::authority::{RelationalSettlementBinding, SettlementPublication};
use crate::publication::data::PublicationSettlementRepairError;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

impl RelationalRuntime {
    /// Complete derived publication for a movement already performed through
    /// the independently borrowable publication port.
    ///
    /// The commit is durable before it is live, so settlement cannot defer: it
    /// installs the derived state and records the commit in the runtime's
    /// canonical log under its original identity. The work itself was installed
    /// in the runtime's pending settlement registry before that movement, so
    /// this is the witness holder taking its turn at the one executor gate, not
    /// the sole owner of the work.
    pub fn settle_performed_publication(
        &self,
        performed: PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> {
        self.publication_authority().settle_performed_publication(
            &self.settlement_binding(),
            performed,
            |input| self.install_settlement_publication(input),
        )
    }

    /// Settle a performed publication retained by this runtime even when the
    /// caller that held its performed witness was lost.
    pub fn repair_pending_publication_settlement(
        &self,
        commit_id: CommitId,
    ) -> Result<RelationalCommitReceipt, PublicationSettlementRepairError> {
        self.publication_authority()
            .repair_pending_publication_settlement(&self.settlement_binding(), commit_id, |input| {
                self.install_settlement_publication(input)
            })
    }

    fn settlement_binding(&self) -> RelationalSettlementBinding<'_> {
        RelationalSettlementBinding {
            runtime_instance_id: self.runtime_instance_id(),
            publication: self.publication_binding(),
            history: &self.history,
            checkpoint: self.durability.checkpoint_selection(),
        }
    }

    fn install_settlement_publication(
        &self,
        input: SettlementPublication,
    ) -> PublishedCommitExecution {
        publish_commit_execution(self, input)
    }
}

impl RelationalRuntime {
    /// The independently borrowable settlement service for this runtime.
    ///
    /// Settlement is the obligation a performed publication transfers to its
    /// caller. The service owns
    /// [`crate::publication::RelationalSettlementPort::settle_performed_publication`] for the
    /// ordinary path and
    /// [`crate::publication::RelationalSettlementPort::repair_pending_publication_settlement`] for a
    /// performed witness that was lost. Dropping performed evidence without settling it is
    /// not success.
    ///
    /// Like
    /// [`RelationalRuntime::preparation_port`](crate::runtime::RelationalRuntime::preparation_port),
    /// this service is obtained from a shared borrow and is
    /// `Clone + Send + Sync`. The runnable owner workflow is
    /// `examples/branch_local_mvcc.rs`.
    pub fn settlement_port(&self) -> crate::publication::RelationalSettlementPort {
        crate::publication::RelationalSettlementPort::new(
            self.runtime_instance_id(),
            self.owner_binding(),
            self.publication_binding(),
            self.state_binding(),
        )
    }
}
