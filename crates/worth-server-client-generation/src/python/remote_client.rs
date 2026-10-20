pub fn render_python_remote_client() -> String {
    r#"# Generated from Worth Server's remote product-client contract. Do not edit.
from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Mapping, Protocol

from .worth_server_protocol import (
    CATALOG_DIGEST,
    WorthServerHttpMethod,
    WorthServerProductOperation,
    WorthServerProductSessionOperation,
    WorthServerRemoteOperation,
    decode_worth_server_response,
)

MAX_ENVELOPE_OVERHEAD_BYTES = 64 * 1024
MAX_PRODUCT_SESSION_RESPONSE_BYTES = 64 * 1024
RESERVED_HEADERS = frozenset(
    {
        "idempotency-key",
        "x-branch-id",
        "x-preview-id",
        "x-principal-id",
        "x-product-session-id",
        "x-tenant-id",
        "x-worth-product-protocol-catalog-digest",
        "x-workspace-id",
        "x-worth-server-instance",
    }
)


class WorthServerRemoteClientError(Exception):
    def __init__(self, code: str, detail: str) -> None:
        super().__init__(detail)
        self.code = code
        self.detail = detail


@dataclass(frozen=True, slots=True)
class WorthServerRemoteTransportRequest:
    method: str
    url: str
    headers: Mapping[str, str]
    query: Mapping[str, str]
    json_body: Mapping[str, object] | None


@dataclass(frozen=True, slots=True)
class WorthServerRemoteTransportResponse:
    status_code: int
    body: bytes


class WorthServerRemoteTransport(Protocol):
    def send(
        self,
        request: WorthServerRemoteTransportRequest,
    ) -> WorthServerRemoteTransportResponse: ...


@dataclass(frozen=True, slots=True)
class WorthServerRemoteRequestScope:
    tenant_identity: str
    workspace_identity: str
    branch_kind: str
    caller_asserted_principal_identity: str | None = None
    branch_identity: str | None = None
    product_session_identity: str | None = None
    basis_digest: str | None = None
    idempotency_key: str | None = None
    server_instance_identity: str | None = None


@dataclass(frozen=True, slots=True)
class WorthServerRemoteResponse:
    status_code: int
    envelope: dict[str, object]
    outcome_kind: str

    @property
    def successful(self) -> bool:
        return self.outcome_kind == "Success"

    def successful_body(self) -> Mapping[str, object]:
        result = self.envelope.get("result")
        if not self.successful or not isinstance(result, Mapping):
            raise WorthServerRemoteClientError(
                "worth_result_unavailable",
                "The Worth response does not contain a successful result.",
            )
        body = result.get("body")
        if not isinstance(body, Mapping):
            raise WorthServerRemoteClientError(
                "worth_result_body_invalid",
                "The Worth success result body is not an object.",
            )
        return body

    def reason_key(self) -> str | None:
        source = self.envelope.get("denial") or self.envelope.get("failure")
        if not isinstance(source, Mapping):
            return None
        reason = source.get("reason_key")
        return reason if isinstance(reason, str) else None


class WorthServerRemoteProductClient:
    def __init__(
        self,
        *,
        server_origin: str,
        transport: WorthServerRemoteTransport,
    ) -> None:
        self._server_origin = _required_origin(server_origin)
        self._transport = transport

    def execute(
        self,
        *,
        operation: WorthServerRemoteOperation,
        scope: WorthServerRemoteRequestScope,
        query: Mapping[str, object] | None = None,
        payload: Mapping[str, object] | None = None,
        caller_headers: Mapping[str, str] | None = None,
    ) -> WorthServerRemoteResponse:
        headers = _request_headers(operation, scope, caller_headers or {})
        request_query = _request_query(operation, scope, query or {})
        request_payload = dict(payload or {})
        if operation.method is WorthServerHttpMethod.GET and request_payload:
            raise WorthServerRemoteClientError(
                "worth_structured_get_unsupported",
                "GET operations cannot carry a structured request body.",
            )
        response = self._transport.send(
            WorthServerRemoteTransportRequest(
                method=operation.method.value,
                url=f"{self._server_origin}{operation.route_path}",
                headers=headers,
                query=request_query,
                json_body=request_payload if operation.method is WorthServerHttpMethod.POST else None,
            )
        )
        if len(response.body) > _response_budget(operation):
            raise WorthServerRemoteClientError(
                "worth_response_too_large",
                "The Worth response exceeded the operation's admitted size.",
            )
        try:
            envelope = json.loads(response.body)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise WorthServerRemoteClientError(
                "worth_response_invalid",
                "The Worth response was not valid JSON.",
            ) from exc
        try:
            outcome = decode_worth_server_response(operation, envelope)
        except ValueError as exc:
            raise WorthServerRemoteClientError(
                str(exc),
                "The Worth response violated its generated protocol contract.",
            ) from exc
        return WorthServerRemoteResponse(
            status_code=response.status_code,
            envelope=outcome.envelope,
            outcome_kind=outcome.kind,
        )


def _request_headers(
    operation: WorthServerRemoteOperation,
    scope: WorthServerRemoteRequestScope,
    caller_headers: Mapping[str, str],
) -> dict[str, str]:
    _require_scope_identity(scope.tenant_identity, "tenant_identity")
    _require_scope_identity(scope.workspace_identity, "workspace_identity")
    normalized_caller_headers = _admit_caller_headers(caller_headers)
    headers = {
        "x-tenant-id": scope.tenant_identity,
        "x-workspace-id": scope.workspace_identity,
        "x-worth-product-protocol-catalog-digest": CATALOG_DIGEST,
        **normalized_caller_headers,
    }
    if scope.caller_asserted_principal_identity is not None:
        _require_scope_identity(
            scope.caller_asserted_principal_identity,
            "caller_asserted_principal_identity",
        )
        headers["x-principal-id"] = scope.caller_asserted_principal_identity
    if scope.branch_kind == "branch" and scope.branch_identity:
        headers["x-branch-id"] = scope.branch_identity
    elif scope.branch_kind == "preview" and scope.branch_identity:
        headers["x-preview-id"] = scope.branch_identity
    elif scope.branch_kind != "main":
        raise WorthServerRemoteClientError(
            "worth_request_scope_invalid",
            "Worth request scope contains an invalid branch target.",
        )
    if operation.requires_product_session:
        _require_scope_identity(scope.product_session_identity, "product_session_identity")
        headers["x-product-session-id"] = scope.product_session_identity
    if operation.requires_server_instance or scope.server_instance_identity is not None:
        _require_scope_identity(scope.server_instance_identity, "server_instance_identity")
        headers["x-worth-server-instance"] = scope.server_instance_identity
    if isinstance(operation, WorthServerProductOperation) and operation.requires_idempotency_key:
        _require_scope_identity(scope.idempotency_key, "idempotency_key")
        headers["idempotency-key"] = scope.idempotency_key
    return headers


def _request_query(
    operation: WorthServerRemoteOperation,
    scope: WorthServerRemoteRequestScope,
    query: Mapping[str, object],
) -> dict[str, str]:
    request_query = {name: _query_value(name, value) for name, value in query.items()}
    if isinstance(operation, WorthServerProductOperation) and operation.requires_basis:
        _require_scope_identity(scope.basis_digest, "basis_digest")
        if "basis" in request_query and request_query["basis"] != scope.basis_digest:
            raise WorthServerRemoteClientError(
                "worth_basis_override_denied",
                "Caller query parameters may not replace the admitted basis.",
            )
        request_query["basis"] = scope.basis_digest
    return request_query


def _admit_caller_headers(headers: Mapping[str, str]) -> dict[str, str]:
    admitted: dict[str, str] = {}
    for name, value in headers.items():
        normalized = name.strip().lower()
        if normalized in RESERVED_HEADERS:
            raise WorthServerRemoteClientError(
                "worth_reserved_header_override_denied",
                f"Caller evidence may not replace reserved header `{normalized}`.",
            )
        if not normalized or not value:
            raise WorthServerRemoteClientError(
                "worth_caller_header_invalid",
                "Caller evidence headers must have non-empty names and values.",
            )
        admitted[normalized] = value
    return admitted


def _query_value(name: str, value: object) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, float) and not math.isfinite(value):
        raise WorthServerRemoteClientError(
            "worth_query_value_invalid",
            f"Query parameter `{name}` must be finite.",
        )
    if isinstance(value, (str, int, float)) and not isinstance(value, bool):
        return str(value)
    raise WorthServerRemoteClientError(
        "worth_query_value_invalid",
        f"Query parameter `{name}` must be a scalar value.",
    )


def _response_budget(operation: WorthServerRemoteOperation) -> int:
    if isinstance(operation, WorthServerProductSessionOperation):
        return MAX_PRODUCT_SESSION_RESPONSE_BYTES
    return operation.result_max_inline_bytes + MAX_ENVELOPE_OVERHEAD_BYTES


def _require_scope_identity(value: str | None, name: str) -> None:
    if not value:
        raise WorthServerRemoteClientError(
            "worth_request_scope_invalid",
            f"Worth request scope requires `{name}`.",
        )


def _required_origin(value: str) -> str:
    normalized = value.strip().rstrip("/")
    if not normalized.startswith(("http://", "https://")):
        raise WorthServerRemoteClientError(
            "worth_server_origin_invalid",
            "server_origin must be an absolute HTTP origin.",
        )
    return normalized


__all__ = [
    "WorthServerRemoteClientError",
    "WorthServerRemoteProductClient",
    "WorthServerRemoteRequestScope",
    "WorthServerRemoteResponse",
    "WorthServerRemoteTransport",
    "WorthServerRemoteTransportRequest",
    "WorthServerRemoteTransportResponse",
]
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::render_python_remote_client;

    #[test]
    fn generated_client_protects_server_owned_headers() {
        let client = render_python_remote_client();
        assert!(client.contains("worth_reserved_header_override_denied"));
        assert!(client.contains("x-worth-product-protocol-catalog-digest"));
    }
}
