//! An installed application's identity as the modules beneath the primary graph see it.
//!
//! The runtime that installs an application implements this; host traits of modules that sit
//! below the primary graph extend it instead of naming the graph.
use worth_query_installation::facade::{
    InstalledOperationCatalog, WorthQueryInstalledApplicationSchema,
};

use super::product_world::WorthQueryProductRuntime;
use super::WorthQueryRuntimeAuthorityIdentity;

/// Hosts are shared across the threads a context crosses, as the runtime itself is.
/// The installed schema and operations, the runtime authority, the product runtime and the
/// runtime clock admissions are judged against.
pub trait WorthQueryApplicationIdentityHost<Schema>: Send + Sync {
    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema>;
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn product_runtime(&self) -> &WorthQueryProductRuntime;
    fn installed_operations(&self) -> &std::sync::Arc<InstalledOperationCatalog>;
    fn authorization_clock(
        &self,
    ) -> &std::sync::Arc<crate::domain_computation::runtime_time::WorthQueryRuntimeClock>;
}
