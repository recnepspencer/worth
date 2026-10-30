use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, Method, Uri},
    response::Response,
    routing::{get, options, post},
    Router,
};
use std::collections::HashMap;

use super::{
    execution_bridge::WorthServerRouteExecutionOutcome,
    request_decoding::{WorthServerRouteBranchTarget, WorthServerRouteTransportRequest},
    response_projection::{semantic_response, transport_denial_response},
    WorthServerOperationRouter, WorthServerRouteAssembly, WorthServerTransportDenial,
};
use crate::transport::{
    WorthServerTransportCallerAdmissionRequest, WorthServerTransportPrincipal,
    WorthServerTransportRequestCancellationGuard,
};

pub(crate) fn project_axum_router(
    assembly: &WorthServerRouteAssembly,
    operation_router: WorthServerOperationRouter,
) -> Router {
    let mut router = Router::new();
    for route in assembly.declared_routes() {
        router = match route.method() {
            "GET" => router.route(route.path(), get(semantic_route_handler)),
            "POST" => router.route(route.path(), post(semantic_route_handler)),
            _ => router,
        };
    }
    for route in assembly.operational_routes() {
        router = match route.method() {
            "GET" => router.route(route.path(), get(operational_route_handler)),
            "OPTIONS" => router.route(route.path(), options(operational_route_handler)),
            _ => router,
        };
    }
    router
        .fallback(unknown_route_handler)
        .with_state(operation_router)
}

#[expect(
    clippy::disallowed_methods,
    reason = "async-runtime async-blocking=1: Tokio runs the synchronous semantic route on its blocking executor."
)]
async fn semantic_route_handler(
    method: Method,
    uri: Uri,
    State(operation_router): State<WorthServerOperationRouter>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let request_lifecycle = WorthServerTransportRequestCancellationGuard::new();
    let Some(bridge) = operation_router.bridge_for(method.as_str(), uri.path()) else {
        return unknown_route_handler().await;
    };
    let caller_request = WorthServerTransportCallerAdmissionRequest::new(
        bridge
            .operation_name()
            .expect("semantic route bridge must name its operation"),
        method.as_str(),
        uri.path(),
        header_pairs(&headers),
        request_lifecycle.cancellation(),
    );
    let transport_request = operation_router
        .caller_admission()
        .admit(&caller_request, optional_header(&headers, "x-principal-id"))
        .and_then(|principal| lower_transport_request(&headers, &query, body.to_vec(), principal));
    let execution = match transport_request {
        Ok(request) => tokio::task::spawn_blocking(move || bridge.execute(request))
            .await
            .map_err(|error| {
                WorthServerTransportDenial::new(
                    super::WorthServerTransportDenialCode::RouteExecutionFailed,
                    format!("semantic route execution task failed: {error}"),
                )
            })
            .and_then(|outcome| outcome),
        Err(denial) => Err(denial),
    };
    match execution {
        Ok(outcome) => semantic_response(outcome),
        Err(denial) => transport_denial_response(denial),
    }
}

async fn operational_route_handler(
    method: Method,
    uri: Uri,
    State(operation_router): State<WorthServerOperationRouter>,
) -> Response {
    let Some(bridge) = operation_router.bridge_for(method.as_str(), uri.path()) else {
        return unknown_route_handler().await;
    };
    match bridge.execute(WorthServerRouteTransportRequest::new(
        "operational",
        "operational",
        "operational",
        WorthServerRouteBranchTarget::Main,
    )) {
        Ok(WorthServerRouteExecutionOutcome::Operational(outcome)) => {
            semantic_response(WorthServerRouteExecutionOutcome::Operational(outcome))
        }
        Ok(other) => semantic_response(other),
        Err(denial) => transport_denial_response(denial),
    }
}

async fn unknown_route_handler() -> Response {
    transport_denial_response(WorthServerTransportDenial::new(
        super::WorthServerTransportDenialCode::UnknownRoute,
        "no declared or operational route matched the request",
    ))
}

fn lower_transport_request(
    headers: &HeaderMap,
    query: &HashMap<String, String>,
    body: Vec<u8>,
    principal: WorthServerTransportPrincipal,
) -> Result<WorthServerRouteTransportRequest, WorthServerTransportDenial> {
    let tenant_id = required_header(headers, "x-tenant-id", "tenant id")?;
    let workspace_id = required_header(headers, "x-workspace-id", "workspace id")?;
    let branch_target = if let Some(preview_id) = optional_header(headers, "x-preview-id") {
        WorthServerRouteBranchTarget::Preview {
            preview_id: preview_id.to_string(),
        }
    } else if let Some(branch_id) = optional_header(headers, "x-branch-id") {
        WorthServerRouteBranchTarget::Branch {
            branch_id: branch_id.to_string(),
        }
    } else {
        WorthServerRouteBranchTarget::Main
    };
    let mut request = WorthServerRouteTransportRequest::new(
        principal.principal_identity(),
        tenant_id,
        workspace_id,
        branch_target,
    );
    if let Some(admitted_caller) = principal.admitted_caller() {
        request = request.with_admitted_transport_caller(admitted_caller.clone());
    }
    for (name, value) in headers {
        if is_sensitive_transport_header(name.as_str()) {
            continue;
        }
        if let Ok(value) = value.to_str() {
            request = request.with_header(name.as_str(), value);
        }
    }
    for (name, value) in query {
        request = request.with_query_pair(name, value);
    }
    if !body.is_empty() {
        request = request.with_raw_body(body, optional_header(headers, "content-type"));
    }
    Ok(request)
}

fn header_pairs(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect()
}

fn is_sensitive_transport_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("authorization")
        || name.eq_ignore_ascii_case("proxy-authorization")
        || name.eq_ignore_ascii_case("x-principal-id")
}

fn required_header<'a>(
    headers: &'a HeaderMap,
    name: &str,
    label: &str,
) -> Result<&'a str, WorthServerTransportDenial> {
    optional_header(headers, name).ok_or_else(|| {
        WorthServerTransportDenial::new(
            match name {
                "x-principal-id" => {
                    super::WorthServerTransportDenialCode::MissingAuthenticatedPrincipalId
                }
                "x-tenant-id" => super::WorthServerTransportDenialCode::MissingTenantId,
                _ => super::WorthServerTransportDenialCode::MissingWorkspaceId,
            },
            format!("compat route requests require `{name}` header for {label}"),
        )
    })
}

fn optional_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::lower_transport_request;
    use crate::transport::WorthServerTransportPrincipal;
    use axum::http::{HeaderMap, HeaderValue};
    use std::collections::HashMap;

    #[test]
    fn lower_transport_request_preserves_original_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert("x-principal-id", HeaderValue::from_static("principal-7"));
        headers.insert("x-tenant-id", HeaderValue::from_static("tenant-a"));
        headers.insert("x-workspace-id", HeaderValue::from_static("workspace-42"));
        headers.insert("content-type", HeaderValue::from_static("text/plain"));

        let request = lower_transport_request(
            &headers,
            &HashMap::new(),
            br#"{"title":"Rename"}"#.to_vec(),
            WorthServerTransportPrincipal::CallerAsserted {
                principal_identity: "principal-7".to_string(),
            },
        )
        .expect("transport request should lower");

        assert_eq!(request.body_content_type(), Some("text/plain"));
    }
}
