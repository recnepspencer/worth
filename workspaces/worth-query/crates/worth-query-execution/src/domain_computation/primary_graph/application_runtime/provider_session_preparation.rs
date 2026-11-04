//! A real installed Home fences a portable grant before constructing live work.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use crate::domain_computation::{
    WorthQueryPreparedProviderSession, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionGrant, WorthQueryProviderSessionReceiver,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The receiving runtime supplies its actual provider and Home witness.
    /// Only a current grant may construct the run, reads and product resources.
    pub fn prepare_provider_session_grant<'run>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
        grant: WorthQueryProviderSessionGrant,
        receiver: impl FnOnce() -> Result<
            WorthQueryProviderSessionReceiver<'run>,
            WorthQueryProviderSessionFailure,
        >,
    ) -> Result<WorthQueryPreparedProviderSession<'run>, WorthQueryProviderSessionFailure> {
        let provider = self
            .primary_graph_authority
            .retain_provider_anchor::<WorthQueryGraphProviderAnchor>()
            .expect("a published primary graph retains its original provider");
        grant.prepare_with_receiver(
            phase,
            Some(self.product_runtime.owner.owner_identity()),
            &provider,
            receiver,
        )
    }
}
