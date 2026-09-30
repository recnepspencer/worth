//! Exact canonical owner settlement for a guarded workflow operation.

use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Correlation selects a terminal candidate; its original publication and
    /// dispatch facts must match the guarded receipt before workflow use.
    pub(in crate::domain_computation::primary_graph) fn resolve_guarded_workflow_external_settlement(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<bool, Denial> {
        let Some(record) = receipt.dispatch_outbox() else {
            return Ok(false);
        };
        if record.inbound().is_none() {
            return Ok(false);
        }
        if receipt.outcome_identity().map(|identity| identity.get())
            != Some(record.outcome_identity())
            || receipt.commit_reference()
                != receipt.committed_product_publication().relational_commit()
        {
            return Err(Denial::WorldPairMismatch);
        }
        match self
            .primary_provider
            .lookup_completed_inbound(record.correlation())?
        {
            Some(terminal)
                if terminal.matches_original_dispatch(
                    record,
                    receipt.commit_reference(),
                    receipt.committed_product_publication().composite_commit(),
                    receipt
                        .committed_product_publication()
                        .product_incarnation(),
                ) =>
            {
                Ok(true)
            }
            Some(_) => Err(Denial::WorldPairMismatch),
            None => Ok(false),
        }
    }
}
