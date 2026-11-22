/// Whole provider result after the sole fallible receipt-axis projection.
pub struct WorthQueryCommittedReceiptProjection {
    provider: super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
    committed_dispatch_outbox:
        super::super::super::super::provider::WorthQueryCommittedDispatchOutboxReceiptSeal,
}

impl WorthQueryCommittedReceiptProjection {
    pub fn resolve(
        provider: super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
    ) -> Result<
        Self,
        super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBindingDenial,
    > {
        let committed_dispatch_outbox = provider
            .commit_evidence()
            .committed_dispatch_outbox()
            .seal_for_receipt()?;
        Ok(Self {
            provider,
            committed_dispatch_outbox,
        })
    }

    pub fn committed_dispatch_outbox(
        &self,
    ) -> Option<&super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>
    {
        self.committed_dispatch_outbox.binding()
    }

    pub fn into_parts(
        self,
    ) -> (
        super::super::super::super::provider::WorthQueryPrimaryGraphCommittedApplication,
        Option<super::super::super::super::provider::WorthQueryCommittedDispatchOutboxBinding>,
    ) {
        (self.provider, self.committed_dispatch_outbox.into_binding())
    }
}
