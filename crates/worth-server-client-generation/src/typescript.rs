use worth_server::WorthServerProductProtocolCatalog;

pub fn render_typescript_product_protocol(catalog: &WorthServerProductProtocolCatalog) -> String {
    let mut output = format!(
        r#"// Generated from Worth Server's product protocol catalog. Do not edit.

export const worthProductProtocolCatalogVersion = {version} as const
export const worthProductProtocolCatalogDigest = {digest:?} as const

export interface WorthServerProductOperationDefinition {{
  name: string
  family: 'read' | 'mutation'
  method: 'GET' | 'POST'
  route: string
  requestSchema: string
  resultSchema: string
  resultSchemaVersion: number
  resultMaxInlineBytes: number
  requiresProductSession: boolean
  requiresBasis: boolean
  requiresIdempotencyKey: boolean
}}

export interface WorthServerProductSessionOperationDefinition {{
  name: string
  method: 'POST'
  route: string
  requestSchema: string
  responseSchema: string
  requiresProductSession: boolean
}}

export type WorthServerOperationOutcome<T> =
  | {{ kind: 'Success'; envelope: WorthServerOperationEnvelope<T>; body: T }}
  | {{ kind: 'Denial'; envelope: WorthServerOperationEnvelope<T>; denial: WorthServerOperationDenial }}
  | {{ kind: 'Failure'; envelope: WorthServerOperationEnvelope<T>; failure: WorthServerOperationFailure }}

export interface WorthServerOperationFailure {{
  reason_key: string
  detail: string
  [key: string]: unknown
}}

export interface WorthServerOperationDenial extends WorthServerOperationFailure {{
  code: string | null
  expected_basis_digest: string | null
  observed_basis_digest: string | null
}}

export interface WorthServerOperationResult<T> {{
  result_key: string
  schema_identity: string
  schema_version: number
  encoding: string
  canonicalization: string
  body: T
  body_digest: string
  artifact_digest: string
  [key: string]: unknown
}}

export interface WorthServerDurableCompletion {{
  disposition: string
  request_digest: string
  completion_digest: string
  next_basis: string
  product_commit_digest: string
  [key: string]: unknown
}}

export interface WorthServerOperationEnvelope<T> {{
  route_kind: 'product_operation'
  operation_name: string
  envelope_kind: 'Success' | 'Denial' | 'Failure'
  canonical_digest: string
  plan_digest: string | null
  result: WorthServerOperationResult<T> | null
  denial: WorthServerOperationDenial | null
  failure: WorthServerOperationFailure | null
  durable_completion: WorthServerDurableCompletion | null
  [key: string]: unknown
}}

export interface WorthServerProductSessionResponse {{
  route_kind: 'product_session'
  product_session_identity: string
  plan_digest: string
}}

export const worthProductOperations = {{
"#,
        version = catalog.schema_version(),
        digest = catalog.catalog_digest(),
    );
    for operation in catalog.operations() {
        output.push_str(&format!(
            "  {:?}: {{\n    name: {:?}, family: {:?}, method: {:?}, route: {:?},\n    requestSchema: {:?}, resultSchema: {:?}, resultSchemaVersion: {}, resultMaxInlineBytes: {},\n    requiresProductSession: {}, requiresBasis: {}, requiresIdempotencyKey: {},\n  }},\n",
            operation.operation_name(),
            operation.operation_name(),
            client_family(operation.operation_family()),
            operation.method(),
            operation.route(),
            operation.request_schema_identity(),
            operation.result_schema_identity(),
            operation.result_schema_version(),
            operation.result_max_inline_bytes(),
            operation.requires_product_session(),
            operation.requires_basis(),
            operation.requires_idempotency_key(),
        ));
    }
    output.push_str("} as const satisfies Record<string, WorthServerProductOperationDefinition>\n\nexport const worthProductSessionOperations = {\n");
    for operation in catalog.product_session_operations() {
        output.push_str(&format!(
            "  {:?}: {{\n    name: {:?}, method: {:?}, route: {:?},\n    requestSchema: {:?}, responseSchema: {:?}, requiresProductSession: {},\n  }},\n",
            operation.operation_name(),
            operation.operation_name(),
            operation.method(),
            operation.route(),
            operation.request_schema_identity(),
            operation.response_schema_identity(),
            operation.requires_product_session(),
        ));
    }
    output.push_str(
        r#"} as const satisfies Record<string, WorthServerProductSessionOperationDefinition>

export function decodeWorthServerProductSessionResponse(value: unknown): WorthServerProductSessionResponse {
  if (
    !isRecord(value) ||
    value.route_kind !== 'product_session' ||
    typeof value.product_session_identity !== 'string' ||
    typeof value.plan_digest !== 'string'
  ) {
    throw new Error('worth_product_session_response_invalid')
  }
  return value as unknown as WorthServerProductSessionResponse
}

export function decodeWorthServerOperationOutcome<T>(
  definition: WorthServerProductOperationDefinition,
  value: unknown,
): WorthServerOperationOutcome<T> {
  if (!isRecord(value) || value.route_kind !== 'product_operation' || value.operation_name !== definition.name) {
    throw new Error('worth_operation_mismatch')
  }
  if (!isEnvelopeMetadata(value)) {
    throw new Error('worth_envelope_invalid')
  }
  const result = value.result
  const denial = value.denial
  const failure = value.failure
  if (value.envelope_kind === 'Success' && isResult(result) && denial == null && failure == null) {
    if (result.schema_identity !== definition.resultSchema || result.schema_version !== definition.resultSchemaVersion) {
      throw new Error('worth_result_schema_mismatch')
    }
    return { kind: 'Success', envelope: value as unknown as WorthServerOperationEnvelope<T>, body: result.body as T }
  }
  if (value.envelope_kind === 'Denial' && result == null && isDenial(denial) && failure == null) {
    return { kind: 'Denial', envelope: value as unknown as WorthServerOperationEnvelope<T>, denial }
  }
  if (value.envelope_kind === 'Failure' && result == null && denial == null && isFailure(failure)) {
    return { kind: 'Failure', envelope: value as unknown as WorthServerOperationEnvelope<T>, failure }
  }
  throw new Error('worth_outcome_invalid')
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function isEnvelopeMetadata(value: Record<string, unknown>): boolean {
  return typeof value.canonical_digest === 'string' &&
    (value.plan_digest === null || typeof value.plan_digest === 'string') &&
    (value.durable_completion === null || isDurableCompletion(value.durable_completion))
}

function isResult(value: unknown): value is WorthServerOperationResult<unknown> {
  return isRecord(value) &&
    typeof value.result_key === 'string' &&
    typeof value.schema_identity === 'string' &&
    Number.isInteger(value.schema_version) &&
    typeof value.encoding === 'string' &&
    typeof value.canonicalization === 'string' &&
    'body' in value &&
    typeof value.body_digest === 'string' &&
    typeof value.artifact_digest === 'string'
}

function isDenial(value: unknown): value is WorthServerOperationDenial {
  return isFailure(value) &&
    isNullableString(value.code) &&
    isNullableString(value.expected_basis_digest) &&
    isNullableString(value.observed_basis_digest)
}

function isFailure(value: unknown): value is WorthServerOperationFailure {
  return isRecord(value) && typeof value.reason_key === 'string' && typeof value.detail === 'string'
}

function isDurableCompletion(value: unknown): value is WorthServerDurableCompletion {
  return isRecord(value) &&
    typeof value.disposition === 'string' &&
    typeof value.request_digest === 'string' &&
    typeof value.completion_digest === 'string' &&
    typeof value.next_basis === 'string' &&
    typeof value.product_commit_digest === 'string'
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === 'string'
}
"#,
    );
    output
}

fn client_family(operation_family: &str) -> &'static str {
    if operation_family.contains("mutation") {
        "mutation"
    } else {
        "read"
    }
}

#[cfg(test)]
mod tests {
    use super::client_family;

    #[test]
    fn protocol_families_lower_to_client_request_classes() {
        assert_eq!(client_family("product-application-mutation"), "mutation");
        assert_eq!(client_family("product-application-read"), "read");
    }
}
