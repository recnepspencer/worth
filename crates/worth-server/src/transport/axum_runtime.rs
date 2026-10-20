use std::io;

use tokio::net::TcpListener;

use crate::{
    declaration_intake::WorthServerDirectDeclarationIntakeFacade,
    runtime::WorthServerRuntime,
    surfaces::compat_http::WorthServerCompatibilityFacade,
    transport::{route_assembly::project_axum_router, WorthServerOperationRouter},
};

pub(crate) async fn serve_runtime(runtime: WorthServerRuntime) -> io::Result<()> {
    let listener =
        TcpListener::bind(runtime.assembly().config().bind_address().socket_addr()).await?;
    runtime.assembly().counters().increment_serve_start_count();
    let _surface_inventory = runtime.assembly().surface_registry().inventory();
    let compat_http = WorthServerCompatibilityFacade::new(
        crate::surfaces::compat_http::WorthServerCompatibilityFacadeParts {
            root: runtime.assembly().surfaces_facade().compat_http(),
            operation_registry: runtime.assembly().operation_registry().clone(),
            product_adapter_registry: runtime.assembly().product_adapter_registry().clone(),
            product_session_registry: runtime.assembly().product_session_registry().clone(),
            request_contexts: runtime.assembly().request_context_facade().clone(),
            middleware: runtime.assembly().middleware_facade().clone(),
            declaration_intake: WorthServerDirectDeclarationIntakeFacade::new(
                runtime.assembly().config().query_handoff().clone(),
            ),
            query_handoff: runtime.assembly().query_handoff_facade().clone(),
            responses: runtime.assembly().response_facade().clone(),
            operator_evidence: runtime.assembly().operator_evidence_facade().clone(),
            idempotency_store: runtime
                .assembly()
                .compat_http_mutation_retry_store()
                .clone(),
            product_operation_retry_store: runtime
                .assembly()
                .product_operation_retry_store()
                .clone(),
            binary_ingress_store: runtime
                .assembly()
                .compat_http_binary_ingress_store()
                .clone(),
            counters: runtime.assembly().counters().clone(),
        },
    );
    let router = project_axum_router(
        runtime.assembly().route_assembly(),
        WorthServerOperationRouter::new(
            runtime
                .assembly()
                .product_session_registry()
                .instance_identity()
                .clone(),
            runtime.assembly().instance_selection(),
            runtime.assembly().route_assembly().clone(),
            compat_http,
            runtime.assembly().transport_caller_admission().clone(),
        ),
    );
    axum::serve(listener, router).await
}
