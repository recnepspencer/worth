//! The runtime's identity and wiring, as every module's host trait sees it.
//!
//! Each module below the application runtime owns a host trait listing what it needs from
//! above; each extends this one, so the runtime's identity is declared once.
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

/// The installed schema, the runtime authority, the product runtime and the graph handle.
pub trait WorthQueryRuntimeIdentityHost<Schema> {
    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema>;
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn product_runtime(&self) -> &WorthQueryProductRuntime;
    fn provider_graph(&self) -> &WorthQueryPrimaryGraphIntegrationHandle;
    fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph>;
}
