//! The runtime arms only its installed provider's grant preparation boundary.
use super::super::{WorthQueryApplicationRuntimeCore, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use crate::domain_computation::{
    WorthQueryProviderSessionGrantControlDenial, WorthQueryProviderSessionGrantControlForTest,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Capture the next authentic grant before any physical provider callback.
    #[doc(hidden)]
    pub fn capture_next_provider_session_grant_for_test(
        &self,
    ) -> Result<
        WorthQueryProviderSessionGrantControlForTest,
        WorthQueryProviderSessionGrantControlDenial,
    > {
        capture_next_grant(&self.runtime)
    }
}

fn capture_next_grant(
    runtime: &WorthQueryApplicationRuntimeCore,
) -> Result<WorthQueryProviderSessionGrantControlForTest, WorthQueryProviderSessionGrantControlDenial>
{
    runtime
        .primary_graph_authority()
        .retain_provider_anchor::<WorthQueryGraphProviderAnchor>()
        .expect("published primary authority retains its original provider")
        .session_grant_control
        .arm()
}
