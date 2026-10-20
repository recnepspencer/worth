use worth_server::WorthServerProductProtocolCatalog;

pub fn render_python_product_protocol(catalog: &WorthServerProductProtocolCatalog) -> String {
    let mut output = format!(
        r#"# Generated from Worth Server's product protocol catalog. Do not edit.
from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import Mapping

CATALOG_VERSION = {version}
CATALOG_DIGEST = {digest:?}


class WorthServerHttpMethod(StrEnum):
    GET = "GET"
    POST = "POST"


@dataclass(frozen=True, slots=True)
class WorthServerProductOperation:
    name: str
    method: WorthServerHttpMethod
    route_path: str
    request_schema: str
    result_schema: str
    result_schema_version: int
    result_max_inline_bytes: int
    requires_product_session: bool
    requires_basis: bool
    requires_idempotency_key: bool


@dataclass(frozen=True, slots=True)
class WorthServerProductSessionOperation:
    name: str
    method: WorthServerHttpMethod
    route_path: str
    request_schema: str
    response_schema: str
    requires_product_session: bool


WorthServerRemoteOperation = WorthServerProductOperation | WorthServerProductSessionOperation


@dataclass(frozen=True, slots=True)
class WorthServerOperationOutcome:
    kind: str
    envelope: dict[str, object]

    @property
    def successful(self) -> bool:
        return self.kind == "Success"


def decode_worth_server_response(
    operation: WorthServerRemoteOperation,
    value: object,
) -> WorthServerOperationOutcome:
    if isinstance(operation, WorthServerProductSessionOperation):
        return _decode_product_session_response(value)
    return _decode_product_operation_outcome(operation, value)


def _decode_product_operation_outcome(
    operation: WorthServerProductOperation,
    value: object,
) -> WorthServerOperationOutcome:
    if not isinstance(value, dict):
        raise ValueError("worth_outcome_invalid")
    if value.get("route_kind") != "product_operation" or value.get("operation_name") != operation.name:
        raise ValueError("worth_operation_mismatch")
    if not _is_envelope_metadata(value):
        raise ValueError("worth_envelope_invalid")
    result = value.get("result")
    denial = value.get("denial")
    failure = value.get("failure")
    if value.get("envelope_kind") == "Success" and _is_result(result) and denial is None and failure is None:
        if result.get("schema_identity") != operation.result_schema or result.get("schema_version") != operation.result_schema_version:
            raise ValueError("worth_result_schema_mismatch")
        return WorthServerOperationOutcome(kind="Success", envelope=value)
    if value.get("envelope_kind") == "Denial" and result is None and _is_denial(denial) and failure is None:
        return WorthServerOperationOutcome(kind="Denial", envelope=value)
    if value.get("envelope_kind") == "Failure" and result is None and denial is None and _is_failure(failure):
        return WorthServerOperationOutcome(kind="Failure", envelope=value)
    raise ValueError("worth_outcome_invalid")


def _decode_product_session_response(value: object) -> WorthServerOperationOutcome:
    if (
        not isinstance(value, dict)
        or value.get("route_kind") != "product_session"
        or not isinstance(value.get("product_session_identity"), str)
        or not isinstance(value.get("plan_digest"), str)
    ):
        raise ValueError("worth_product_session_response_invalid")
    return WorthServerOperationOutcome(kind="Success", envelope=value)


def _is_envelope_metadata(value: Mapping[str, object]) -> bool:
    return (
        isinstance(value.get("canonical_digest"), str)
        and "plan_digest" in value
        and (value.get("plan_digest") is None or isinstance(value.get("plan_digest"), str))
        and "durable_completion" in value
        and (
            value.get("durable_completion") is None
            or _is_durable_completion(value.get("durable_completion"))
        )
    )


def _is_result(value: object) -> bool:
    return (
        isinstance(value, Mapping)
        and isinstance(value.get("result_key"), str)
        and isinstance(value.get("schema_identity"), str)
        and isinstance(value.get("schema_version"), int)
        and not isinstance(value.get("schema_version"), bool)
        and isinstance(value.get("encoding"), str)
        and isinstance(value.get("canonicalization"), str)
        and "body" in value
        and isinstance(value.get("body_digest"), str)
        and isinstance(value.get("artifact_digest"), str)
    )


def _is_denial(value: object) -> bool:
    return (
        _is_failure(value)
        and isinstance(value, Mapping)
        and "code" in value
        and _is_nullable_string(value.get("code"))
        and "expected_basis_digest" in value
        and _is_nullable_string(value.get("expected_basis_digest"))
        and "observed_basis_digest" in value
        and _is_nullable_string(value.get("observed_basis_digest"))
    )


def _is_failure(value: object) -> bool:
    return (
        isinstance(value, Mapping)
        and isinstance(value.get("reason_key"), str)
        and isinstance(value.get("detail"), str)
    )


def _is_durable_completion(value: object) -> bool:
    return (
        isinstance(value, Mapping)
        and isinstance(value.get("disposition"), str)
        and isinstance(value.get("request_digest"), str)
        and isinstance(value.get("completion_digest"), str)
        and isinstance(value.get("next_basis"), str)
        and isinstance(value.get("product_commit_digest"), str)
    )


def _is_nullable_string(value: object) -> bool:
    return value is None or isinstance(value, str)


PRODUCT_OPERATIONS: dict[str, WorthServerProductOperation] = {{
"#,
        version = catalog.schema_version(),
        digest = catalog.catalog_digest(),
    );
    for operation in catalog.operations() {
        output.push_str(&format!(
            "    {:?}: WorthServerProductOperation(\n        name={:?}, method=WorthServerHttpMethod.{}, route_path={:?},\n        request_schema={:?}, result_schema={:?}, result_schema_version={}, result_max_inline_bytes={},\n        requires_product_session={}, requires_basis={}, requires_idempotency_key={},\n    ),\n",
            operation.operation_name(),
            operation.operation_name(),
            operation.method(),
            operation.route(),
            operation.request_schema_identity(),
            operation.result_schema_identity(),
            operation.result_schema_version(),
            operation.result_max_inline_bytes(),
            python_bool(operation.requires_product_session()),
            python_bool(operation.requires_basis()),
            python_bool(operation.requires_idempotency_key()),
        ));
    }
    output.push_str(
        "}\n\nPRODUCT_SESSION_OPERATIONS: dict[str, WorthServerProductSessionOperation] = {\n",
    );
    for operation in catalog.product_session_operations() {
        output.push_str(&format!(
            "    {:?}: WorthServerProductSessionOperation(\n        name={:?}, method=WorthServerHttpMethod.{}, route_path={:?},\n        request_schema={:?}, response_schema={:?}, requires_product_session={},\n    ),\n",
            operation.operation_name(),
            operation.operation_name(),
            operation.method(),
            operation.route(),
            operation.request_schema_identity(),
            operation.response_schema_identity(),
            python_bool(operation.requires_product_session()),
        ));
    }
    output.push_str(
        r#"}

__all__ = [
    "CATALOG_DIGEST",
    "CATALOG_VERSION",
    "PRODUCT_OPERATIONS",
    "PRODUCT_SESSION_OPERATIONS",
    "WorthServerHttpMethod",
    "WorthServerOperationOutcome",
    "WorthServerProductOperation",
    "WorthServerProductSessionOperation",
    "WorthServerRemoteOperation",
    "decode_worth_server_response",
]
"#,
    );
    output
}

fn python_bool(value: bool) -> &'static str {
    if value {
        "True"
    } else {
        "False"
    }
}

#[cfg(test)]
mod tests {
    use super::python_bool;

    #[test]
    fn boolean_literals_are_python_values() {
        assert_eq!(python_bool(true), "True");
        assert_eq!(python_bool(false), "False");
    }
}
