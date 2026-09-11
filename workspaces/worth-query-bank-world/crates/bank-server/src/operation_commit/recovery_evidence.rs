//! Crate-private execution evidence retained only for governed Bank recovery.

use bank_domain::schema::BankSchema;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryCommittedDispatchOutboxReadDenial, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
};

#[derive(Clone)]
pub(crate) struct BankCommitRecoveryEvidence {
    execution: WorthQueryApplicationCommitReceipt,
}

impl BankCommitRecoveryEvidence {
    pub(super) const fn from_execution(execution: WorthQueryApplicationCommitReceipt) -> Self {
        Self { execution }
    }

    pub(crate) fn observe_dispatch_outbox(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    ) -> Result<
        Option<WorthQueryCommittedDispatchOutboxObservation>,
        WorthQueryCommittedDispatchOutboxReadDenial,
    > {
        runtime.observe_committed_dispatch_outbox(&self.execution)
    }

    pub(crate) fn mint_recovery_handle(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    ) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
        runtime.mint_recovery_handle(&self.execution)
    }
}
