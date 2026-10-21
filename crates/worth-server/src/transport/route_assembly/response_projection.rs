use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use super::{
    execution_bridge::{WorthServerOperationalRouteOutcome, WorthServerRouteExecutionOutcome},
    WorthServerTransportDenial, WorthServerTransportDenialCode,
};

pub(super) fn semantic_response(outcome: WorthServerRouteExecutionOutcome) -> Response {
    match outcome {
        WorthServerRouteExecutionOutcome::ProductOperation(operation) => {
            product_operation_response(*operation)
        }
        WorthServerRouteExecutionOutcome::ProductSession(coordination) => response_with_headers(
            Json(json!({
                "route_kind": "product_session",
                "product_session_identity": coordination.session().identity().as_str(),
                "server_instance": coordination.session().server_instance().as_str(),
                "basis": coordination.session().basis_digest(),
                "plan_digest": coordination.plan().canonical_digest(),
            })),
            semantic_headers(
                "product_session",
                Some(coordination.plan().command().operation_name()),
                Some(coordination.plan().canonical_digest()),
                None,
                Some(coordination.scheduler_admission().scheduler_lane()),
            ),
        ),
        WorthServerRouteExecutionOutcome::Operational(outcome) => response_with_headers(
            operational_response(outcome),
            operational_headers("operational"),
        ),
    }
}

fn product_operation_response(operation: crate::WorthServerCompletedProductOperation) -> Response {
    let result = match operation.outcome() {
        crate::WorthServerProductOperationOutcome::Success(success) => {
            let artifact = success.result_artifact();
            Some(json!({
                "result_key": success.result_key(),
                "schema_identity": artifact.contract().schema().identity(),
                "schema_version": artifact.contract().schema().version(),
                "encoding": artifact.contract().encoding().as_str(),
                "canonicalization": artifact.contract().canonicalization().as_str(),
                "body": artifact.body().value(),
                "body_digest": artifact.body_digest(),
                "artifact_digest": artifact.artifact_digest(),
            }))
        }
        crate::WorthServerProductOperationOutcome::Denied(_)
        | crate::WorthServerProductOperationOutcome::Failed(_) => None,
    };
    let durable_completion = operation.durable_mutation_receipt().map(|receipt| {
        json!({
            "disposition": receipt.disposition().as_str(),
            "request_digest": receipt.request_digest(),
            "completion_digest": receipt.completion_digest(),
            "next_basis": receipt.next_basis().value(),
            "product_commit_digest": receipt.product_commit_digest(),
        })
    });
    let denial = match operation.outcome() {
        crate::WorthServerProductOperationOutcome::Denied(denial) => Some(json!({
            "reason_key": denial.reason_key(),
            "detail": denial.detail(),
            "code": denial.facts().map(|facts| format!("{:?}", facts.code())),
            "expected_basis_digest": denial.facts().and_then(|facts| facts.expected_basis_digest()),
            "observed_basis_digest": denial.facts().and_then(|facts| facts.observed_basis_digest()),
        })),
        crate::WorthServerProductOperationOutcome::Success(_)
        | crate::WorthServerProductOperationOutcome::Failed(_) => None,
    };
    let failure = match operation.outcome() {
        crate::WorthServerProductOperationOutcome::Failed(failure) => Some(json!({
            "reason_key": failure.reason_key(),
            "detail": failure.detail(),
        })),
        crate::WorthServerProductOperationOutcome::Success(_)
        | crate::WorthServerProductOperationOutcome::Denied(_) => None,
    };
    response_with_headers(
        Json(json!({
            "route_kind": "product_operation",
            "operation_name": operation.envelope().operation_name(),
            "envelope_kind": format!("{:?}", operation.envelope().kind()),
            "canonical_digest": operation.envelope().canonical_digest(),
            "plan_digest": operation.plan().map(|plan| plan.canonical_digest()),
            "result": result,
            "denial": denial,
            "failure": failure,
            "durable_completion": durable_completion,
        })),
        semantic_headers(
            "product_operation",
            Some(operation.envelope().operation_name()),
            operation.plan().map(|plan| plan.canonical_digest()),
            Some(operation.envelope().canonical_digest()),
            operation
                .scheduler_admission()
                .map(|admission| admission.scheduler_lane()),
        ),
    )
}

pub(super) fn operational_response(
    outcome: WorthServerOperationalRouteOutcome,
) -> Json<serde_json::Value> {
    Json(json!({
        "route_kind": "operational",
        "kind": format!("{:?}", outcome.kind()),
        "path": outcome.path(),
    }))
}

pub(super) fn transport_denial_response(denial: WorthServerTransportDenial) -> Response {
    let status = match denial.code() {
        WorthServerTransportDenialCode::MalformedJson => StatusCode::BAD_REQUEST,
        WorthServerTransportDenialCode::OversizedBody => StatusCode::PAYLOAD_TOO_LARGE,
        WorthServerTransportDenialCode::UnsupportedContentType => {
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        }
        WorthServerTransportDenialCode::UnknownRoute => StatusCode::NOT_FOUND,
        WorthServerTransportDenialCode::RouteExecutionFailed => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };
    response_with_headers(
        (
            status,
            Json(json!({
                "transport_denial_code": format!("{:?}", denial.code()),
                "transport_denial_reason_key": denial.reason_key(),
                "requested_server_instance": denial.requested_server_instance(),
                "actual_server_instance": denial.actual_server_instance(),
                "detail": denial.detail(),
                "product_operation_denial": denial.product_operation_denial().map(super::product_surface_denial::project),
            })),
        ),
        transport_denial_headers(denial.code()),
    )
}

fn response_with_headers<T>(response: T, headers: HeaderMap) -> Response
where
    T: IntoResponse,
{
    let mut response = response.into_response();
    response.headers_mut().extend(headers);
    response
}

fn semantic_headers(
    route_kind: &str,
    operation_name: Option<&str>,
    plan_digest: Option<&str>,
    envelope_digest: Option<&str>,
    scheduler_lane: Option<&str>,
) -> HeaderMap {
    let mut headers = base_headers(route_kind, true);
    insert_optional_header(&mut headers, "x-Worth-operation-name", operation_name);
    insert_optional_header(&mut headers, "x-Worth-plan-digest", plan_digest);
    insert_optional_header(&mut headers, "x-Worth-envelope-digest", envelope_digest);
    insert_optional_header(&mut headers, "x-Worth-scheduler-lane", scheduler_lane);
    headers
}

fn operational_headers(route_kind: &str) -> HeaderMap {
    base_headers(route_kind, false)
}

fn transport_denial_headers(code: WorthServerTransportDenialCode) -> HeaderMap {
    let mut headers = base_headers("transport_denial", false);
    insert_header(
        &mut headers,
        "x-Worth-transport-denial-code",
        &format!("{code:?}"),
    );
    headers
}

fn base_headers(route_kind: &str, entered_semantic_runtime: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    insert_header(&mut headers, "x-Worth-route-kind", route_kind);
    insert_header(
        &mut headers,
        "x-Worth-semantic-runtime-entered",
        if entered_semantic_runtime {
            "true"
        } else {
            "false"
        },
    );
    headers
}

fn insert_optional_header(headers: &mut HeaderMap, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        insert_header(headers, name, value);
    }
}

fn insert_header(headers: &mut HeaderMap, name: &str, value: &str) {
    if let (Ok(name), Ok(value)) = (
        HeaderName::from_bytes(name.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        headers.insert(name, value);
    }
}
