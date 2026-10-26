use crate::{
    request_context::DiagnosticRichnessProfile, WorthServerCanonicalFilename,
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerOperationFamily, WorthServerOperationRequestInput,
};

use super::{
    WorthServerOperationInputEnvelope, WorthServerOperationRequestDenial,
    WorthServerOperationRequestDenialCode,
};

pub(crate) fn lower_compat_http_request_input(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    operation_family: WorthServerOperationFamily,
    operation_name: &str,
    payload_envelope: Option<WorthServerOperationInputEnvelope>,
) -> Result<WorthServerOperationRequestInput, WorthServerOperationRequestDenial> {
    let diagnostics_profile = prepared_request
        .admission()
        .request_context()
        .diagnostics_profile();
    validate_compatibility_operation_binding(
        prepared_request,
        operation_name,
        diagnostics_profile,
    )?;
    let mut builder = WorthServerOperationRequestInput::builder()
        .with_operation_family(operation_family)
        .with_operation_name(operation_name);
    if operation_family != WorthServerOperationFamily::QueryDirectRead
        && let Some(basis_digest) = lower_basis_digest(prepared_request, diagnostics_profile)?
    {
        builder = builder.with_basis_digest(basis_digest);
    }
    if let Some(idempotency_key) = lower_idempotency_key(prepared_request, diagnostics_profile)? {
        builder = builder.with_idempotency_key(idempotency_key);
    }
    if let Some(payload_envelope) = payload_envelope {
        builder = builder.with_payload_envelope(payload_envelope);
    }
    Ok(builder.build())
}

pub(crate) fn lower_browser_origin(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
) -> Result<Option<String>, WorthServerOperationRequestDenial> {
    let diagnostics_profile = prepared_request
        .admission()
        .request_context()
        .diagnostics_profile();
    let Some(values) = prepared_request
        .request_contract()
        .canonical_headers()
        .values("origin")
    else {
        return Ok(None);
    };
    if values.len() != 1 {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidBrowserOrigin,
            diagnostics_profile,
            "operation request admits at most one canonical browser `Origin` header",
        ));
    }
    let origin = values[0].trim();
    if origin.is_empty() {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidBrowserOrigin,
            diagnostics_profile,
            "operation request browser `Origin` header may not be blank",
        ));
    }
    Ok(Some(origin.to_string()))
}

fn lower_basis_digest(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> Result<Option<String>, WorthServerOperationRequestDenial> {
    let values = prepared_request
        .request_contract()
        .normalized_query_pairs()
        .iter()
        .filter(|(name, _)| name == "basis")
        .map(|(_, value)| value.trim())
        .collect::<Vec<_>>();
    if values.len() > 1 {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidBasisDigest,
            diagnostics_profile,
            "operation request admits at most one canonical basis query parameter",
        ));
    }
    let Some(value) = values.first() else {
        return Ok(None);
    };
    if value.is_empty() {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidBasisDigest,
            diagnostics_profile,
            "operation request basis digest may not be blank",
        ));
    }
    Ok(Some((*value).to_string()))
}

fn lower_idempotency_key(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> Result<Option<String>, WorthServerOperationRequestDenial> {
    let Some(values) = prepared_request
        .request_contract()
        .canonical_headers()
        .values("idempotency-key")
    else {
        return Ok(None);
    };
    if values.len() != 1 {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidIdempotencyKey,
            diagnostics_profile,
            "operation request requires a single canonical `idempotency-key` header value",
        ));
    }
    let value = values[0].trim();
    if value.is_empty() {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidIdempotencyKey,
            diagnostics_profile,
            "operation request `idempotency-key` may not be blank",
        ));
    }
    Ok(Some(value.to_string()))
}

pub(crate) fn validate_compatibility_operation_binding(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> Result<(), WorthServerOperationRequestDenial> {
    let claimed = WorthServerCanonicalFilename::admit(
        operation_name,
        diagnostics_profile,
        crate::WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
    )
    .map_err(|denial| {
        WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidOperationName,
            diagnostics_profile,
            denial.detail(),
        )
    })?;
    let route_stem = match prepared_request.request_contract().route_family() {
        WorthServerCompatHttpRouteFamily::Read => "/compat/reads/",
        WorthServerCompatHttpRouteFamily::Query => "/compat/queries/",
        WorthServerCompatHttpRouteFamily::Mutation => "/compat/mutations/",
        WorthServerCompatHttpRouteFamily::Streaming => "/compat/streams/",
        WorthServerCompatHttpRouteFamily::Upload => "/compat/uploads/",
        WorthServerCompatHttpRouteFamily::Download => "/compat/downloads/",
        WorthServerCompatHttpRouteFamily::Preflight => {
            return Err(WorthServerOperationRequestDenial::new(
                WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid,
                diagnostics_profile,
                "preflight requests do not lower to canonical operation requests",
            ));
        }
    };
    let Some(remainder) = prepared_request
        .request_contract()
        .normalized_path()
        .strip_prefix(route_stem)
    else {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid,
            diagnostics_profile,
            format!(
                "compatibility path `{}` did not match route stem `{route_stem}`",
                prepared_request.request_contract().normalized_path()
            ),
        ));
    };
    if remainder.is_empty() || remainder.contains('/') {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid,
            diagnostics_profile,
            format!(
                "compatibility path `{}` must carry exactly one operation segment after `{route_stem}`",
                prepared_request.request_contract().normalized_path()
            ),
        ));
    }
    let expected = WorthServerCanonicalFilename::admit(
        remainder,
        diagnostics_profile,
        crate::WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
    )
    .map_err(|denial| {
        WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid,
            diagnostics_profile,
            denial.detail(),
        )
    })?;
    if expected.canonical() != claimed.canonical() {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid,
            diagnostics_profile,
            format!(
                "compatibility operation name `{}` did not match the external request path identity `{}`",
                claimed.original(),
                expected.original(),
            ),
        ));
    }
    Ok(())
}
