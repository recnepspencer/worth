//! The runtime's identity and wiring, as every module's host trait sees it.
//!
//! Each module below the application runtime owns a host trait listing what it needs from
//! above; each extends this one, so the runtime's identity is declared once. The part that
//! does not name the graph is the execution runtime's application identity.
use crate::domain_computation::execution_runtime::WorthQueryApplicationIdentityHost;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

/// The application identity plus the graph handle, the graph and the installed bridge.
pub trait WorthQueryRuntimeIdentityHost<Schema>: WorthQueryApplicationIdentityHost<Schema> {
    fn provider_graph(&self) -> &WorthQueryPrimaryGraphIntegrationHandle;
    fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph>;
    /// Temporary bridge while the modules above still take the concrete runtime; removed when
    /// the last of them takes its own host.
    fn application_runtime(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>;
    fn bridge(&self) -> &crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge;
}
