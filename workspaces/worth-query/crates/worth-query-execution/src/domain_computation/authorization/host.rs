//! What `authorization` needs from the runtime that composes it.
use std::sync::Arc;

use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::primary_graph::product_operation::{
    WorthQueryProductObservationSource, WorthQueryProductSecurityBasis,
};
use crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;

/// The runtime's identity plus the installed authorization registry, its clock, the graph-work
/// support and the product security bases authorization admits against.
pub trait AuthorizationHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::primary_graph::PrincipalCurrentnessCaptureHost<Schema>
{
    /// This host as the trait object `authorization` is handed.
    fn authorization_host(&self) -> &dyn AuthorizationHost<Schema>;
    fn authorization(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry>;
    fn authorization_clock(&self) -> &Arc<WorthQueryRuntimeClock>;
    fn installed_packages(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex;
    fn graph_work_provider_identity(&self) -> &str;
    fn graph_work_resource_support(&self) -> WorthQueryExecutionResourceSupportSnapshot;
    fn graph_work_resource_support_ref(&self) -> &WorthQueryExecutionResourceSupportSnapshot;
    fn authentication_is_expired(&self, valid_until: std::time::Instant) -> bool;
    fn admit_product_security_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &dyn WorthQueryProductObservationSource,
    ) -> Result<
        WorthQueryProductSecurityBasis<'static>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    >;
    fn admit_capability_read_security_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &dyn WorthQueryProductObservationSource,
        prepared: &PreparedSelectedReadIndexes,
    ) -> Result<
        WorthQueryProductSecurityBasis<'static>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    >;
}
