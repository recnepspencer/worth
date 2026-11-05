/// Whole provider result after the sole fallible receipt-axis projection.
pub(in crate::domain_computation::primary_graph) struct WorthQueryCommittedReceiptProjection {
    provider: super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
    committed_dispatch_outbox:
        super::super::super::super::provider::WorthQueryCommittedDispatchOutboxReceiptSeal,
}

impl WorthQueryCommittedReceiptProjection {
    pub(in crate::domain_computation::primary_graph) fn resolve(
        provider: super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
    ) -> Result<
        Self,
        crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial,
    >{
        let committed_dispatch_outbox = provider
            .commit_evidence()
            .committed_dispatch_outbox()
            .seal_for_receipt()?;
        Ok(Self {
            provider,
            committed_dispatch_outbox,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox(
        &self,
    ) -> Option<&crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBinding>
{
        self.committed_dispatch_outbox.binding()
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
        Option<crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBinding>,
    ){
        (self.provider, self.committed_dispatch_outbox.into_binding())
    }
}
