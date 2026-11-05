//! The runtime composes `authorization`.
use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::authorization::host::AuthorizationHost;
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::primary_graph::product_operation::{
    WorthQueryProductObservationSource, WorthQueryProductSecurityBasis,
};
use crate::domain_computation::primary_graph::selected_read_indexes::PreparedSelectedReadIndexes;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;

impl<Schema: ApplicationSchema> AuthorizationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn authorization_host(&self) -> &dyn AuthorizationHost<Schema> {
        self
    }
    fn authorization(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry> {
        &self.runtime.authorization
    }
    fn installed_packages(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex {
        self.runtime.installed_packages()
    }
    fn graph_work_provider_identity(&self) -> &str {
        Self::graph_work_provider_identity(self)
    }
    fn graph_work_resource_support(&self) -> WorthQueryExecutionResourceSupportSnapshot {
        Self::graph_work_resource_support(self)
    }
    fn graph_work_resource_support_ref(&self) -> &WorthQueryExecutionResourceSupportSnapshot {
        Self::graph_work_resource_support_ref(self)
    }
    fn authentication_is_expired(&self, valid_until: std::time::Instant) -> bool {
        Self::authentication_is_expired(self, valid_until)
    }
    fn admit_product_security_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &dyn WorthQueryProductObservationSource,
    ) -> Result<
        WorthQueryProductSecurityBasis<'static>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        Self::admit_product_security_basis(self, execution, product)
    }
    fn admit_capability_read_security_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &dyn WorthQueryProductObservationSource,
        prepared: &PreparedSelectedReadIndexes,
    ) -> Result<
        WorthQueryProductSecurityBasis<'static>,
        crate::basis::WorthQueryProductBranchAdmissionDenial,
    > {
        Self::admit_capability_read_security_basis(self, execution, product, prepared)
    }
}
