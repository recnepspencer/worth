use std::ops::Deref;
use worth_query_execution_runtime::facade::execution_runtime as rt;
use worth_query_installation::facade::{
    WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndexDenial,
};

pub use worth_query_execution_query::facade::domain_computation::execution_runtime::{
    product_world, source_invalidation, WorthQueryApplicationQueryResourceProfile,
    WorthQueryBranchCoordinationResourceProfile, WorthQueryExecutionInstallationCommitDenial,
    WorthQueryExecutionRuntime, WorthQueryInvalidationResources, WorthQueryOutputDemandLimits,
    WorthQueryOutputDemandResourceProfile, WorthQueryRuntimeAuthorityIdentity,
};

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

    pub fn output_demand_resources(
        self,
        profile: WorthQueryOutputDemandResourceProfile,
    ) -> Self {
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
            runtime: WorthQueryExecutionRuntime::from_inner(runtime),
            authority: WorthQueryExecutionInstallationAuthority(authority),
        })
    }
}

impl Default for WorthQueryExecutionRuntimeInstaller {
    fn default() -> Self {
        Self::new()
    }
}
