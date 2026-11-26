//! Shared runtime custody for a whole-result live subscription that lives
//! beside the owner of its installed runtime instead of borrowing it.
use std::sync::Arc;

use super::WorthQuerySharedLiveRuntime;
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::application_runtime::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphApplicationRuntimeInner,
};
use crate::domain_computation::primary_graph::product_operation::SharedPrimaryRuntime;
use worth_query_declaration::facade::{
    application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
};

/// Keeps an installed runtime alive for live subscriptions moved onto it.
///
/// Issued only by [`WorthQueryProgramApplicationRuntime::live_keepalive`].
/// While any clone is held the runtime's clean close refuses
/// `RuntimeKeepaliveRetained`; dropping every clone releases the hold.
#[derive(Clone)]
pub struct WorthQueryLiveRuntimeKeepalive<Schema> {
    application: Arc<dyn SharedPrimaryRuntime<Schema>>,
    live: Arc<dyn WorthQuerySharedLiveRuntime<Schema>>,
}

impl<Schema> WorthQueryLiveRuntimeKeepalive<Schema> {
    /// The installed runtime this custody keeps alive.
    pub fn application(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application.runtime()
    }

    /// The same custody as a live lease's runtime backing.
    pub fn live_backing(&self) -> Arc<dyn WorthQuerySharedLiveRuntime<Schema>> {
        Arc::clone(&self.live)
    }
}

impl<S, P> WorthQuerySharedLiveRuntime<S> for WorthQueryProgramApplicationRuntime<S, P>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    fn live_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<S> {
        self.backing.runtime().inner()
    }
}

impl<S, P> WorthQueryProgramApplicationRuntime<S, P>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S> + 'static,
{
    /// Shared custody of this installed runtime for owned live subscriptions.
    pub fn live_keepalive(&self) -> WorthQueryLiveRuntimeKeepalive<S> {
        let shared = Arc::new(self.clone());
        WorthQueryLiveRuntimeKeepalive {
            application: shared.clone(),
            live: shared,
        }
    }
}
