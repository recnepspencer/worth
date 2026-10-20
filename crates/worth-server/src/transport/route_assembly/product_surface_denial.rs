//! Descriptive projection of the retained owner refusal, never a performed-operation envelope.
use crate::WorthServerProductOperationSurfaceDenial;
use serde_json::{json, Value};

pub(super) fn project(denial: &WorthServerProductOperationSurfaceDenial) -> Value {
    let facts = denial.facts();
    json!({
        "schema_identity": "worth-server-product-surface-denial.v1",
        "schema_version": 1,
        "code": format!("{:?}", denial.code()),
        "readiness_code": facts.and_then(|facts| facts.readiness_code()).map(|code| format!("{code:?}")),
        "expected_basis_digest": facts.and_then(|facts| facts.expected_basis_digest()),
        "observed_basis_digest": facts.and_then(|facts| facts.observed_basis_digest()),
        "execution_boundary": facts.and_then(|facts| facts.execution_boundary()).map(|boundary| format!("{boundary:?}")),
        "session_denial_code": facts.and_then(|facts| facts.session_denial_code()).map(|code| format!("{code:?}")),
        "rebase_required": facts.and_then(|facts| facts.rebase_required()).is_some(),
        "idempotency_conflict": facts.and_then(|facts| facts.idempotency_conflict()).map(|conflict| json!({
            "idempotency_key": conflict.idempotency_key(),
            "conflicting_request_digest": conflict.conflicting_request_digest(),
            "bound_request_digest": conflict.bound_request_digest(),
            "canonical_digest": conflict.canonical_digest(),
        })),
        // Describes retained recovery custody; these bytes do not mint its authority.
        "recovery": facts.and_then(|facts| facts.recovery_handle()).map(|recovery| json!({
            "operation_name": recovery.operation_name(),
            "tenant_id": recovery.tenant_id(),
            "workspace_id": recovery.workspace_id(),
            "authority_scope": recovery.authority_scope().value(),
            "recovery_identity": recovery.recovery_identity(),
            "request_digest": recovery.request_digest(),
            "canonical_digest": recovery.canonical_digest(),
        })),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        WorthServerOperationReadinessDenial, WorthServerOperationReadinessDenialCode,
        WorthServerOperationReadinessDenialFacts, WorthServerProductOperationSurfaceDenial,
        WorthServerTransportDenial, WorthServerTransportDenialCode,
    };

    #[tokio::test]
    async fn stale_surface_refusal_survives_transport_without_claiming_execution() {
        let readiness = WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::PreconditionFailed,
            "opaque detail without any parseable basis names",
        )
        .with_facts(
            WorthServerOperationReadinessDenialFacts::default()
                .with_basis_mismatch("basis:original", "basis:current"),
        );
        let surface = WorthServerProductOperationSurfaceDenial::from_readiness_denial(readiness);
        let transport = WorthServerTransportDenial::from_product_operation(surface.clone());
        assert_eq!(
            transport.code(),
            WorthServerTransportDenialCode::ProductOperationDenied
        );
        assert_eq!(transport.product_operation_denial(), Some(&surface));
        let response = super::super::response_projection::transport_denial_response(transport);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["transport_denial_code"], "ProductOperationDenied");
        let facts = &body["product_operation_denial"];
        assert_eq!(
            facts["schema_identity"],
            "worth-server-product-surface-denial.v1"
        );
        assert_eq!(facts["schema_version"], 1);
        assert_eq!(facts["code"], "ReadinessDenied");
        assert_eq!(facts["readiness_code"], "PreconditionFailed");
        assert_eq!(facts["expected_basis_digest"], "basis:original");
        assert_eq!(facts["observed_basis_digest"], "basis:current");
        assert_eq!(
            facts["execution_boundary"],
            "RejectedBeforeAdapterExecution"
        );
        assert_eq!(facts["rebase_required"], true);
        assert!(facts["recovery"].is_null());
        assert!(body.get("canonical_digest").is_none());
        assert!(body.get("result").is_none());
    }
}
