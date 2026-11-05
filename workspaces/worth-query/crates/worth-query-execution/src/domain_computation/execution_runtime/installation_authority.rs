use {
    super::WorthQueryExecutionRuntime,
    crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity,
};

/// Opaque authority retained by the composition root that installed one
/// execution runtime.
///
/// Runtime identities are descriptive. Possession of this value is what
/// authorizes installed provider closure and provider-call binding.
pub struct WorthQueryExecutionInstallationAuthority {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    installation_runtime: worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity,
}

impl WorthQueryExecutionInstallationAuthority {
    pub(crate) fn new(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        installation_runtime: worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity,
    ) -> Self {
        Self {
            runtime_authority,
            installation_runtime,
        }
    }

    pub(crate) fn belongs_to(&self, runtime: &WorthQueryExecutionRuntime) -> bool {
        self.runtime_authority == runtime.authority_identity()
    }

    pub(crate) fn installation_runtime(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity {
        &self.installation_runtime
    }
}

/// Compiler-total result of installing one execution runtime.
pub struct WorthQueryExecutionRuntimeInstallation {
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
}

impl WorthQueryExecutionRuntimeInstallation {
    pub(crate) fn new(
        runtime: WorthQueryExecutionRuntime,
        installation_runtime: worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity,
    ) -> Self {
        let authority = WorthQueryExecutionInstallationAuthority::new(
            runtime.authority_identity(),
            installation_runtime,
        );
        Self { runtime, authority }
    }

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
