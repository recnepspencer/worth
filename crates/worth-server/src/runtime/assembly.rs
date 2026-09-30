use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionAuthority;

use crate::{
    config::WorthServerConfig,
    diagnostics::WorthServerCounters,
    middleware::WorthServerMiddlewareFacade,
    operation_registry::WorthServerOperationRegistry,
    operator_evidence::WorthServerOperatorEvidenceFacade,
    product_adapter::WorthServerProductAdapterRegistry,
    product_operation_contract::WorthServerStoredProductOperation,
    product_session::{
        default_product_session_clock, SharedProductSessionTerminationObserver,
        WorthServerProductSessionClock, WorthServerProductSessionRegistry,
    },
    query_handoff::WorthServerQueryHandoffFacade,
    registration::WorthServerSurfaceRegistry,
    request_context::WorthServerRequestContextFacade,
    response::WorthServerResponseFacade,
    surfaces::{
        compat_http::{WorthServerStoredBinaryIngress, WorthServerStoredCompatibilityMutation},
        WorthServerSurfacesFacade,
    },
    transport::WorthServerRouteAssembly,
    WorthServerTransportCallerVerifier,
};

#[derive(Debug)]
pub(crate) struct WorthServerRuntimeAssembly {
    config: WorthServerConfig,
    surface_registry: WorthServerSurfaceRegistry,
    operation_registry: WorthServerOperationRegistry,
    product_adapter_registry: WorthServerProductAdapterRegistry,
    surfaces_facade: WorthServerSurfacesFacade,
    middleware_facade: WorthServerMiddlewareFacade,
    operator_evidence_facade: WorthServerOperatorEvidenceFacade,
    query_handoff_facade: WorthServerQueryHandoffFacade,
    response_facade: WorthServerResponseFacade,
    request_context_facade: WorthServerRequestContextFacade,
    route_assembly: WorthServerRouteAssembly,
    counters: Arc<WorthServerCounters>,
    product_session_registry: WorthServerProductSessionRegistry,
    transport_caller_admission: crate::transport::WorthServerTransportCallerAdmission,
    compat_http_mutation_retry_store:
        Arc<Mutex<HashMap<String, WorthServerStoredCompatibilityMutation>>>,
    product_operation_retry_store: Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
    compat_http_binary_ingress_store: Arc<Mutex<HashMap<String, WorthServerStoredBinaryIngress>>>,
    execution_authority: Option<Arc<ExecutionAuthority>>,
}

pub(crate) struct WorthServerRuntimeAssemblyParts {
    pub(crate) config: WorthServerConfig,
    pub(crate) surface_registry: WorthServerSurfaceRegistry,
    pub(crate) operation_registry: WorthServerOperationRegistry,
    pub(crate) product_adapter_registry: WorthServerProductAdapterRegistry,
    pub(crate) route_assembly: WorthServerRouteAssembly,
    pub(crate) counters: Arc<WorthServerCounters>,
    pub(crate) product_session_clock: Option<Arc<dyn WorthServerProductSessionClock>>,
    pub(crate) product_session_termination_observers: Vec<SharedProductSessionTerminationObserver>,
    pub(crate) transport_caller_verifier: Option<Arc<dyn WorthServerTransportCallerVerifier>>,
    pub(crate) execution_authority: Option<Arc<ExecutionAuthority>>,
}

impl WorthServerRuntimeAssembly {
    pub(crate) fn new(parts: WorthServerRuntimeAssemblyParts) -> Self {
        let WorthServerRuntimeAssemblyParts {
            config,
            surface_registry,
            operation_registry,
            product_adapter_registry,
            route_assembly,
            counters,
            product_session_clock,
            product_session_termination_observers,
            transport_caller_verifier,
            execution_authority,
        } = parts;
        let surfaces_facade = WorthServerSurfacesFacade::new(&surface_registry);
        let middleware_facade = WorthServerMiddlewareFacade::new(config.middleware().clone());
        let operator_evidence_facade =
            WorthServerOperatorEvidenceFacade::new(config.operator_evidence().clone());
        let query_handoff_facade =
            WorthServerQueryHandoffFacade::new(config.query_handoff().clone());
        let response_facade = WorthServerResponseFacade::new(config.response().clone());
        let request_context_facade =
            WorthServerRequestContextFacade::new(config.request_context().clone());
        let product_session_registry = WorthServerProductSessionRegistry::new(
            counters.clone(),
            product_session_clock.unwrap_or_else(default_product_session_clock),
            product_session_termination_observers,
        );
        let transport_caller_admission =
            crate::transport::WorthServerTransportCallerAdmission::new(transport_caller_verifier);
        Self {
            config,
            surface_registry,
            operation_registry,
            product_adapter_registry,
            surfaces_facade,
            middleware_facade,
            operator_evidence_facade,
            query_handoff_facade,
            response_facade,
            request_context_facade,
            route_assembly,
            counters,
            product_session_registry,
            transport_caller_admission,
            compat_http_mutation_retry_store: Arc::new(Mutex::new(HashMap::new())),
            product_operation_retry_store: Arc::new(Mutex::new(HashMap::new())),
            compat_http_binary_ingress_store: Arc::new(Mutex::new(HashMap::new())),
            execution_authority,
        }
    }

    pub(crate) fn config(&self) -> &WorthServerConfig {
        &self.config
    }

    pub(crate) fn execution_authority(&self) -> Option<&ExecutionAuthority> {
        self.execution_authority.as_deref()
    }

    pub(crate) fn surface_registry(&self) -> &WorthServerSurfaceRegistry {
        &self.surface_registry
    }

    pub(crate) fn request_context_facade(&self) -> &WorthServerRequestContextFacade {
        &self.request_context_facade
    }

    pub(crate) fn operation_registry(&self) -> &WorthServerOperationRegistry {
        &self.operation_registry
    }

    pub(crate) fn surfaces_facade(&self) -> &WorthServerSurfacesFacade {
        &self.surfaces_facade
    }

    pub(crate) fn product_adapter_registry(&self) -> &WorthServerProductAdapterRegistry {
        &self.product_adapter_registry
    }

    pub(crate) fn middleware_facade(&self) -> &WorthServerMiddlewareFacade {
        &self.middleware_facade
    }

    pub(crate) fn operator_evidence_facade(&self) -> &WorthServerOperatorEvidenceFacade {
        &self.operator_evidence_facade
    }

    pub(crate) fn query_handoff_facade(&self) -> &WorthServerQueryHandoffFacade {
        &self.query_handoff_facade
    }

    pub(crate) fn response_facade(&self) -> &WorthServerResponseFacade {
        &self.response_facade
    }

    pub(crate) fn counters(&self) -> &Arc<WorthServerCounters> {
        &self.counters
    }

    pub(crate) fn route_assembly(&self) -> &WorthServerRouteAssembly {
        &self.route_assembly
    }

    pub(crate) fn product_session_registry(&self) -> &WorthServerProductSessionRegistry {
        &self.product_session_registry
    }

    pub(crate) fn transport_caller_admission(
        &self,
    ) -> &crate::transport::WorthServerTransportCallerAdmission {
        &self.transport_caller_admission
    }

    pub(crate) fn compat_http_mutation_retry_store(
        &self,
    ) -> &Arc<Mutex<HashMap<String, WorthServerStoredCompatibilityMutation>>> {
        &self.compat_http_mutation_retry_store
    }

    pub(crate) fn compat_http_binary_ingress_store(
        &self,
    ) -> &Arc<Mutex<HashMap<String, WorthServerStoredBinaryIngress>>> {
        &self.compat_http_binary_ingress_store
    }

    pub(crate) fn product_operation_retry_store(
        &self,
    ) -> &Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>> {
        &self.product_operation_retry_store
    }
}
