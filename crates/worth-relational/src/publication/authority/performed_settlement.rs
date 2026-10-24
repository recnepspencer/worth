use std::sync::Arc;

use crate::publication::data::PublicationSettlementRepairError;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CommitResult, TransactionCommitError};

use super::pending_settlement::{RelationalSettlementCompletion, RelationalSettlementResultOwner};

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
        performed: crate::mvcc::PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> {
        let (positioned, record) = performed.into_settlement_parts();
        debug_assert_eq!(positioned.envelope().commit.commit_id, record.commit_id());
        if record.runtime_instance_id() != self.runtime_instance_id() {
            return Err(TransactionCommitError::publication_denied(
                crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: record.runtime_instance_id(),
                },
            ));
        }
        let completion = self
            .execute_pending_settlement(&record, RelationalSettlementResultOwner::Witness)
            .expect("a performed witness exists only after its reservation is authorized");
        match completion {
            RelationalSettlementCompletion::Performed { committed, .. } => Ok(committed),
            RelationalSettlementCompletion::Repeated { committed, .. } => {
                let retained = committed.expect(
                    "a receipt-only executor retains the commit result for its live witness",
                );
                Ok(Arc::try_unwrap(retained).unwrap_or_else(|shared| shared.as_ref().clone()))
            }
        }
    }

    /// Settle a performed publication retained by this runtime even when the
    /// caller that held its performed witness was lost.
    pub fn repair_pending_publication_settlement(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> Result<crate::history::data::RelationalCommitReceipt, PublicationSettlementRepairError>
    {
        let Some(record) = self.publication_binding().pending_settlement(commit_id) else {
            if self.publication_binding().settlement_admission_is_closed() {
                return Err(PublicationSettlementRepairError::OwnerUnavailable {
                    runtime_instance_id: self.runtime_instance_id(),
                });
            }
            return self.settled_receipt_without_record(commit_id);
        };
        if record.runtime_instance_id() != self.runtime_instance_id() {
            return Err(PublicationSettlementRepairError::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
                actual_runtime_instance_id: record.runtime_instance_id(),
            });
        }
        self.execute_pending_settlement(&record, RelationalSettlementResultOwner::Runtime)
            .map(|completion| completion.receipt().clone())
            .map_err(|_| PublicationSettlementRepairError::SettlementInProgress { commit_id })
    }
}
