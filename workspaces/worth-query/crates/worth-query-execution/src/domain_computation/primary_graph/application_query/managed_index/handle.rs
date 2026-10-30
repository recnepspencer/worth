//! Cleanup and status for the same runtime-owned registration used by queries.
use super::{
    denial::{ReceiverCause, WorthQueryManagedIndexDenial},
    execution::ProviderExecutionDenial,
    retention::{ManagedIndexState, WorthQueryManagedIndexLifecycle},
};
use crate::domain_computation::primary_graph::application_query::derived_view::ManagedDerivedViewRegistry;
use std::sync::{Arc, Weak};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

/// A Query-issued registration handle, with no provider root or read permission.
///
/// Repeated opens for the same association share this registration. Closing any
/// sibling disposes it for all siblings; a later open creates a new registration.
/// Dropping a handle keeps the runtime-owned cache. Use `close` to release it;
/// final runtime disposal also releases all registrations.
pub struct WorthQueryManagedIndex<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    pub(super) state: Arc<ManagedIndexState<S, I>>,
    pub(super) id: u64,
    pub(super) registry: Weak<ManagedDerivedViewRegistry>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> Clone
    for WorthQueryManagedIndex<S, I>
{
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            id: self.id,
            registry: Weak::clone(&self.registry),
        }
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> WorthQueryManagedIndex<S, I> {
    /// Cache status only; this does not reauthorize or check fresh Source availability.
    pub fn lifecycle(&self) -> WorthQueryManagedIndexLifecycle {
        self.state.lifecycle()
    }

    /// Releases the current image. An authenticated explicit rebuild is required
    /// before queries can use this registration again.
    pub fn evict(&self) -> Result<(), WorthQueryManagedIndexDenial> {
        self.state.evict().map_err(|cause| {
            WorthQueryManagedIndexDenial::new(
                ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause)),
                None,
            )
        })
    }

    /// Irreversibly disposes and unregisters this registration for every sibling.
    /// Cleanup performs no source read or provider callback.
    pub fn close(&self) {
        self.state.close();
        if let Some(registry) = self.registry.upgrade() {
            registry.unregister(self.id);
        }
    }
}
