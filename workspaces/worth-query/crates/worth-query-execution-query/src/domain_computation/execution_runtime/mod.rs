use std::ops::{Deref, DerefMut};
use worth_query_execution_runtime::facade::execution_runtime as rt;
use worth_query_execution_runtime::facade::ports::ExecutionRuntimeInternalPort as _;
use worth_query_installation::facade::{
    WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndexDenial,
};

pub mod product_world;
pub use rt::source_invalidation;
pub use rt::{
    WorthQueryApplicationQueryResourceProfile, WorthQueryBranchCoordinationResourceProfile,
    WorthQueryExecutionInstallationCommitDenial, WorthQueryInvalidationResources,
    WorthQueryOutputDemandLimits, WorthQueryOutputDemandResourceProfile,
    WorthQueryRuntimeAuthorityIdentity,
};

/// Execution-owned root for one installed Query operating world.
#[repr(transparent)]
pub struct WorthQueryExecutionRuntime(pub(crate) rt::WorthQueryExecutionRuntime);

impl Deref for WorthQueryExecutionRuntime {
    type Target = rt::WorthQueryExecutionRuntime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WorthQueryExecutionRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl WorthQueryExecutionRuntime {
    pub(crate) fn from_inner(inner: rt::WorthQueryExecutionRuntime) -> Self {
        Self(inner)
    }

    pub fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        self.0
            .primary_graph_any()
            .and_then(|any| any.downcast_ref())
    }

    pub(crate) fn primary_graph_mut(
        &mut self,
    ) -> Option<&mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        self.0
            .primary_graph_any_mut()
            .and_then(|any| any.downcast_mut())
    }

    pub(crate) fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>
    {
        self.primary_graph().map(
            crate::domain_computation::primary_graph::WorthQueryPrimaryGraph::integration_handle,
        )
    }

    pub(crate) fn install_primary_graph(
        &mut self,
        graph: crate::domain_computation::primary_graph::WorthQueryPrimaryGraph,
    ) {
        self.0.install_primary_graph_any(Box::new(graph));
    }

    pub fn installed_packages(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex {
        self.0.installed_packages()
    }

    pub fn commit_successor_installation(
        &mut self,
        successor: std::sync::Arc<
            worth_query_installation::facade::WorthQueryInstalledPackageIndex,
        >,
    ) -> Result<(), WorthQueryExecutionInstallationCommitDenial> {
        self.0.commit_successor_installation(successor)
    }

    pub fn replace_rebuilt_installation(
        &mut self,
        rebuilt: std::sync::Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>,
    ) -> Result<(), WorthQueryExecutionInstallationCommitDenial> {
        self.0.replace_rebuilt_installation(rebuilt)
    }
}

/// Opaque authority retained by the composition root that installed one
/// execution runtime.
#[repr(transparent)]
pub struct WorthQueryExecutionInstallationAuthority(
    pub(crate) rt::WorthQueryExecutionInstallationAuthority,
);

impl Deref for WorthQueryExecutionInstallationAuthority {
    type Target = rt::WorthQueryExecutionInstallationAuthority;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Compiler-total result of installing one execution runtime.
pub struct WorthQueryExecutionRuntimeInstallation {
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
}

impl WorthQueryExecutionRuntimeInstallation {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryExecutionRuntime,
        WorthQueryExecutionInstallationAuthority,
    ) {
        (self.runtime, self.authority)
    }

    pub fn runtime(&self) -> &WorthQueryExecutionRuntime {
        &self.runtime
    }
}

/// Move-only construction authority for one execution runtime.
pub struct WorthQueryExecutionRuntimeInstaller(rt::WorthQueryExecutionRuntimeInstaller);

impl WorthQueryExecutionRuntimeInstaller {
    pub fn new() -> Self {
        Self(rt::WorthQueryExecutionRuntimeInstaller::new())
    }

    pub fn application_query_resources(
        self,
        profile: WorthQueryApplicationQueryResourceProfile,
    ) -> Self {
        Self(self.0.application_query_resources(profile))
    }

    pub fn output_demand_resources(self, profile: WorthQueryOutputDemandResourceProfile) -> Self {
        Self(self.0.output_demand_resources(profile))
    }

    pub fn branch_coordination_resources(
        self,
        profile: WorthQueryBranchCoordinationResourceProfile,
    ) -> Self {
        Self(self.0.branch_coordination_resources(profile))
    }

    pub fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.0.authority_identity()
    }

    pub fn installation_runtime(&self) -> &WorthQueryInstallationRuntimeIdentity {
        self.0.installation_runtime()
    }

    pub fn install(
        self,
        generation: WorthQueryInstallationGeneration,
        packages: impl IntoIterator<Item = WorthQueryAdmittedPortableDomainPackage>,
    ) -> Result<WorthQueryExecutionRuntimeInstallation, WorthQueryInstalledPackageIndexDenial> {
        let (runtime, authority) = self.0.install(generation, packages)?.into_parts();
        Ok(WorthQueryExecutionRuntimeInstallation {
            runtime: WorthQueryExecutionRuntime(runtime),
            authority: WorthQueryExecutionInstallationAuthority(authority),
        })
    }
}

impl Default for WorthQueryExecutionRuntimeInstaller {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::ports::ExecutionRuntimePrimaryGraphPort for WorthQueryExecutionRuntime {
    fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>
    {
        Self::retain_primary_graph_integration_handle(self)
    }
}

/// Crate-boundary port for `WorthQueryExecutionRuntime` consumed by
/// `worth-query-execution`. Never exported on `worth-query-execution::facade`
/// or Host.
pub trait QueryExecutionRuntimeInternalPort: Sized {
    fn from_inner(inner: rt::WorthQueryExecutionRuntime) -> Self;
    fn primary_graph_mut(
        &mut self,
    ) -> Option<&mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraph>;
    fn install_primary_graph(
        &mut self,
        graph: crate::domain_computation::primary_graph::WorthQueryPrimaryGraph,
    );
}

impl QueryExecutionRuntimeInternalPort for WorthQueryExecutionRuntime {
    fn from_inner(inner: rt::WorthQueryExecutionRuntime) -> Self {
        Self::from_inner(inner)
    }

    fn primary_graph_mut(
        &mut self,
    ) -> Option<&mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        Self::primary_graph_mut(self)
    }

    fn install_primary_graph(
        &mut self,
        graph: crate::domain_computation::primary_graph::WorthQueryPrimaryGraph,
    ) {
        Self::install_primary_graph(self, graph)
    }
}

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
