use std::io;

use crate::{
    declaration_intake::WorthServerDirectDeclarationIntakeFacade,
    diagnostics::WorthServerCounterSnapshot,
    middleware::WorthServerMiddlewareFacade,
    operation_admission::WorthServerOperationAdmissionFacade,
    operation_planning::WorthServerOperationPlanner,
    operation_readiness::WorthServerOperationReadinessFacade,
    operation_registry::{WorthServerOperationInventory, WorthServerOperationRegistry},
    operation_request::WorthServerOperationRequestFacade,
    operation_runtime_certification::WorthServerProductOperationRuntimeCertificationFacade,
    operation_scheduler::WorthServerOperationScheduler,
    operator_evidence::WorthServerOperatorEvidenceFacade,
    product_adapter::{
        WorthServerProductAdapterRegistrationReceipt, WorthServerProductOperationRuntime,
    },
    product_protocol_catalog::{
        project_product_protocol_catalog, WorthServerProductProtocolCatalog,
        WorthServerProductProtocolCatalogError,
    },
    product_session::WorthServerProductSessionRegistry,
    query_dependency_audit::WorthServerQueryDependencyAuditFacade,
    query_handoff::WorthServerQueryHandoffFacade,
    registration::WorthServerSurfaceInventory,
    request_context::WorthServerRequestContextFacade,
    response::WorthServerResponseFacade,
    runtime::WorthServerRuntime,
    surfaces::compat_http::WorthServerCompatibilityFacade,
    surfaces::WorthServerSurfacesFacade,
    transport::serve_runtime,
    transport::{
        WorthServerOperationRouter, WorthServerProjectedRouter, WorthServerRouteInventory,
    },
    worth_native::WorthServerWorthNativeFacade,
};

use super::builder::WorthServerBuilder;

#[derive(Debug)]
pub struct WorthServer {
    runtime: WorthServerRuntime,
}

impl WorthServer {
    pub fn builder() -> WorthServerBuilder {
        WorthServerBuilder::default()
    }

    pub(crate) fn new(runtime: WorthServerRuntime) -> Self {
        Self { runtime }
    }

    pub fn surface_inventory(&self) -> WorthServerSurfaceInventory {
        self.runtime.assembly().surface_registry().inventory()
    }

    pub fn counters(&self) -> WorthServerCounterSnapshot {
        self.runtime.assembly().counters().snapshot()
    }

    pub fn operation_inventory(&self) -> WorthServerOperationInventory {
        self.runtime.assembly().operation_registry().inventory()
    }

    /// Borrow the authority installed by the host for request lease admission.
    pub fn execution_authority(&self) -> Option<&worth_execution::ExecutionAuthority> {
        self.runtime.assembly().execution_authority()
    }

    pub fn route_inventory(&self) -> WorthServerRouteInventory {
        self.runtime.assembly().route_assembly().inventory().clone()
    }

    pub fn operation_registry(&self) -> WorthServerOperationRegistry {
        self.runtime.assembly().operation_registry().clone()
    }

    pub fn operation_requests(&self) -> WorthServerOperationRequestFacade {
        WorthServerOperationRequestFacade::new(self.runtime.assembly().operation_registry().clone())
    }

    pub fn operation_admissions(&self) -> WorthServerOperationAdmissionFacade {
        WorthServerOperationAdmissionFacade::with_operation_registry(
            self.runtime.assembly().operation_registry().clone(),
        )
    }

    pub fn operation_readiness(&self) -> WorthServerOperationReadinessFacade {
        WorthServerOperationReadinessFacade::with_operation_registry(
            self.runtime.assembly().operation_registry().clone(),
        )
    }

    pub fn operation_planner(&self) -> WorthServerOperationPlanner {
        WorthServerOperationPlanner::with_operation_registry(
            self.runtime.assembly().config().query_handoff().clone(),
            self.runtime.assembly().operation_registry().clone(),
        )
    }

    pub fn operation_scheduler(&self) -> WorthServerOperationScheduler {
        WorthServerOperationScheduler::new(self.responses())
    }

    pub fn product_adapter_inventory(&self) -> &[WorthServerProductAdapterRegistrationReceipt] {
        self.runtime
            .assembly()
            .product_adapter_registry()
            .receipts()
    }

    pub fn product_protocol_catalog(
        &self,
    ) -> Result<WorthServerProductProtocolCatalog, WorthServerProductProtocolCatalogError> {
        project_product_protocol_catalog(
            self.runtime.assembly().product_adapter_registry(),
            self.runtime.assembly().route_assembly().inventory(),
        )
    }

    pub fn product_operation_runtime(&self) -> WorthServerProductOperationRuntime {
        WorthServerProductOperationRuntime::new(
            self.runtime.assembly().operation_registry().clone(),
            self.runtime.assembly().product_adapter_registry().clone(),
            self.runtime.assembly().config().query_handoff().clone(),
            self.runtime.assembly().product_session_registry().clone(),
            self.runtime
                .assembly()
                .product_operation_retry_store()
                .clone(),
            self.runtime.assembly().counters().clone(),
        )
    }

    pub fn product_session_registry(&self) -> WorthServerProductSessionRegistry {
        self.runtime.assembly().product_session_registry().clone()
    }

    pub fn operation_runtime_certification(
        &self,
    ) -> WorthServerProductOperationRuntimeCertificationFacade {
        WorthServerProductOperationRuntimeCertificationFacade::new(
            self.query_dependency_audit(),
            self.operation_inventory(),
            self.route_inventory(),
            self.product_adapter_inventory().to_vec(),
        )
    }

    pub fn request_contexts(&self) -> WorthServerRequestContextFacade {
        self.runtime.assembly().request_context_facade().clone()
    }

    pub fn middleware(&self) -> WorthServerMiddlewareFacade {
        self.runtime.assembly().middleware_facade().clone()
    }

    pub fn query_handoff(&self) -> WorthServerQueryHandoffFacade {
        self.runtime.assembly().query_handoff_facade().clone()
    }

    pub fn query_dependency_audit(&self) -> WorthServerQueryDependencyAuditFacade {
        WorthServerQueryDependencyAuditFacade::new(
            self.request_contexts(),
            self.runtime.assembly().config().query_handoff().clone(),
        )
    }

    pub fn operator_evidence(&self) -> WorthServerOperatorEvidenceFacade {
        self.runtime.assembly().operator_evidence_facade().clone()
    }

    pub fn responses(&self) -> WorthServerResponseFacade {
        self.runtime.assembly().response_facade().clone()
    }

    pub fn surfaces(&self) -> WorthServerSurfacesFacade {
        self.runtime.assembly().surfaces_facade().clone()
    }

    pub fn worth_native(&self) -> WorthServerWorthNativeFacade {
        WorthServerWorthNativeFacade::new(crate::worth_native::WorthServerWorthNativeFacadeParts {
            root: self.runtime.assembly().surfaces_facade().worth_native(),
            operation_registry: self.runtime.assembly().operation_registry().clone(),
            product_adapter_registry: self.runtime.assembly().product_adapter_registry().clone(),
            product_session_registry: self.runtime.assembly().product_session_registry().clone(),
            product_operation_retry_store: self
                .runtime
                .assembly()
                .product_operation_retry_store()
                .clone(),
            counters: self.runtime.assembly().counters().clone(),
            request_contexts: self.request_contexts(),
            middleware: self.middleware(),
            declaration_intake: WorthServerDirectDeclarationIntakeFacade::new(
                self.runtime.assembly().config().query_handoff().clone(),
            ),
            query_handoff: self.query_handoff(),
            responses: self.responses(),
        })
    }

    pub fn compat_http(&self) -> WorthServerCompatibilityFacade {
        WorthServerCompatibilityFacade::new(
            crate::surfaces::compat_http::WorthServerCompatibilityFacadeParts {
                root: self.runtime.assembly().surfaces_facade().compat_http(),
                operation_registry: self.runtime.assembly().operation_registry().clone(),
                product_adapter_registry: self
                    .runtime
                    .assembly()
                    .product_adapter_registry()
                    .clone(),
                product_session_registry: self
                    .runtime
                    .assembly()
                    .product_session_registry()
                    .clone(),
                request_contexts: self.request_contexts(),
                middleware: self.middleware(),
                declaration_intake: WorthServerDirectDeclarationIntakeFacade::new(
                    self.runtime.assembly().config().query_handoff().clone(),
                ),
                query_handoff: self.query_handoff(),
                responses: self.responses(),
                operator_evidence: self.operator_evidence(),
                idempotency_store: self
                    .runtime
                    .assembly()
                    .compat_http_mutation_retry_store()
                    .clone(),
                product_operation_retry_store: self
                    .runtime
                    .assembly()
                    .product_operation_retry_store()
                    .clone(),
                binary_ingress_store: self
                    .runtime
                    .assembly()
                    .compat_http_binary_ingress_store()
                    .clone(),
                counters: self.runtime.assembly().counters().clone(),
            },
        )
    }

    pub fn operation_router(&self) -> WorthServerOperationRouter {
        WorthServerOperationRouter::new(
            self.runtime.assembly().route_assembly().clone(),
            self.compat_http(),
            self.runtime.assembly().transport_caller_admission().clone(),
        )
    }

    pub fn projected_router(&self) -> WorthServerProjectedRouter {
        WorthServerProjectedRouter::new(crate::transport::project_axum_router(
            self.runtime.assembly().route_assembly(),
            self.operation_router(),
        ))
    }

    pub async fn serve(self) -> io::Result<()> {
        serve_runtime(self.runtime).await
    }
}
