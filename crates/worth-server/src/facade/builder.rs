use std::sync::Arc;
use worth_execution::ExecutionAuthority;

use crate::{
    config::WorthServerConfig,
    diagnostics::WorthServerCounters,
    operation_registry::{
        WorthServerOperationRegistration, WorthServerOperationRegistry,
        WorthServerOperationRegistryError,
    },
    product_adapter::{
        WorthServerProductAdapterRegistry, WorthServerProductAdapterRegistryError,
        WorthServerProductApplicationAdapterRegistration,
    },
    product_session::{
        SharedProductSessionTerminationObserver, WorthServerProductSessionClock,
        WorthServerProductSessionTerminationObserver,
    },
    registration::{
        WorthServerSurfaceRegistration, WorthServerSurfaceRegistry, WorthServerSurfaceRegistryError,
    },
    runtime::{WorthServerRuntime, WorthServerRuntimeAssembly, WorthServerRuntimeAssemblyParts},
    transport::WorthServerRouteAssemblyError,
};

use super::server::WorthServer;

#[derive(Debug, Default)]
pub struct WorthServerBuilder {
    config: Option<WorthServerConfig>,
    surface_registrations: Vec<WorthServerSurfaceRegistration>,
    operation_registrations: Vec<WorthServerOperationRegistration>,
    product_adapter_registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
    product_session_clock: Option<Arc<dyn WorthServerProductSessionClock>>,
    product_session_termination_observers: Vec<SharedProductSessionTerminationObserver>,
    transport_caller_verifier: Option<Arc<dyn crate::WorthServerTransportCallerVerifier>>,
    product_operation_authorizer: Option<Arc<dyn crate::WorthServerProductOperationAuthorizer>>,
    execution_authority: Option<Arc<ExecutionAuthority>>,
}

impl WorthServerBuilder {
    pub fn with_config(mut self, config: WorthServerConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn register_surface(mut self, registration: WorthServerSurfaceRegistration) -> Self {
        self.surface_registrations.push(registration);
        self
    }

    pub fn register_operation(mut self, registration: WorthServerOperationRegistration) -> Self {
        self.operation_registrations.push(registration);
        self
    }

    pub fn register_operations(
        mut self,
        registrations: impl IntoIterator<Item = WorthServerOperationRegistration>,
    ) -> Self {
        self.operation_registrations.extend(registrations);
        self
    }

    pub fn register_product_adapter(
        mut self,
        registration: WorthServerProductApplicationAdapterRegistration,
    ) -> Self {
        self.product_adapter_registrations.push(registration);
        self
    }

    pub fn register_product_adapters(
        mut self,
        registrations: impl IntoIterator<Item = WorthServerProductApplicationAdapterRegistration>,
    ) -> Self {
        self.product_adapter_registrations.extend(registrations);
        self
    }

    pub fn with_product_session_clock(
        mut self,
        product_session_clock: Arc<dyn WorthServerProductSessionClock>,
    ) -> Self {
        self.product_session_clock = Some(product_session_clock);
        self
    }

    pub fn observe_product_session_termination(
        mut self,
        observer: Arc<dyn WorthServerProductSessionTerminationObserver>,
    ) -> Self {
        self.product_session_termination_observers.push(observer);
        self
    }

    pub fn with_transport_caller_verifier(
        mut self,
        verifier: Arc<dyn crate::WorthServerTransportCallerVerifier>,
    ) -> Self {
        self.transport_caller_verifier = Some(verifier);
        self
    }

    pub fn with_product_operation_authorizer(
        mut self,
        authorizer: Arc<dyn crate::WorthServerProductOperationAuthorizer>,
    ) -> Self {
        self.product_operation_authorizer = Some(authorizer);
        self
    }

    /// Share the host's process execution authority with Server requests.
    pub fn with_execution_authority(mut self, authority: Arc<ExecutionAuthority>) -> Self {
        self.execution_authority = Some(authority);
        self
    }

    pub fn build(self) -> Result<WorthServer, WorthServerBuildError> {
        let config = self.config.ok_or(WorthServerBuildError::MissingConfig)?;
        let counters = Arc::new(WorthServerCounters::default());
        let surface_registry =
            WorthServerSurfaceRegistry::build(self.surface_registrations, counters.as_ref())
                .map_err(WorthServerBuildError::InvalidSurfaceRegistry)?;
        let operation_registry =
            WorthServerOperationRegistry::build(self.operation_registrations, counters.as_ref())
                .map_err(WorthServerBuildError::InvalidOperationRegistry)?;
        let product_adapter_registry =
            WorthServerProductAdapterRegistry::build(self.product_adapter_registrations)
                .map_err(WorthServerBuildError::InvalidProductAdapterRegistry)?
                .with_operation_authorizer(self.product_operation_authorizer);
        let route_assembly = crate::transport::WorthServerRouteAssembly::assemble(
            &crate::surfaces::CompatHttpSurfaceRoot::new(&surface_registry),
            &operation_registry,
            &product_adapter_registry,
        )
        .map_err(WorthServerBuildError::InvalidRouteAssembly)?;
        let assembly = WorthServerRuntimeAssembly::new(WorthServerRuntimeAssemblyParts {
            config,
            surface_registry,
            operation_registry,
            product_adapter_registry,
            route_assembly,
            counters,
            product_session_clock: self.product_session_clock,
            product_session_termination_observers: self.product_session_termination_observers,
            transport_caller_verifier: self.transport_caller_verifier,
            execution_authority: self.execution_authority,
        });
        let runtime = WorthServerRuntime::from_assembly(assembly);
        Ok(WorthServer::new(runtime))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerBuildError {
    MissingConfig,
    InvalidSurfaceRegistry(WorthServerSurfaceRegistryError),
    InvalidOperationRegistry(WorthServerOperationRegistryError),
    InvalidProductAdapterRegistry(WorthServerProductAdapterRegistryError),
    InvalidRouteAssembly(WorthServerRouteAssemblyError),
}
