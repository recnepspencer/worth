//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    #[doc(hidden)]
    pub fn deliver_output_checkpoint_with_finish_for_test(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        producer_identity: &str,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Delivery,
        ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
        selected: bool,
        prepared: Option<PreparedSelectedDelivery<'_>>,
        finish: impl FnOnce(
            Checkpoint,
            Option<WorthQueryOutputDemandDenial>,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        Self::deliver_output_checkpoint_with_finish(
            self,
            phase,
            producer_identity,
            receipt,
            delivery,
            ready_backing,
            selected,
            prepared,
            finish,
        )
    }
}
