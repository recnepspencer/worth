use crate::{
    request_context::DiagnosticRichnessProfile, WorthServerCanonicalFilename,
    WorthServerOperationInputEnvelope, WorthServerOperationRequestDenial,
    WorthServerOperationRequestDenialCode, WorthServerResolvedRequestContext,
};

pub(super) fn canonicalize_product_basis_digest_if_needed(value: &str) -> String {
    crate::WorthServerProductOperationBaseDigest::canonicalize_text(value)
        .unwrap_or_else(|_| value.trim().to_string())
}

pub(super) fn missing_family(
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerOperationRequestDenial {
    WorthServerOperationRequestDenial::new(
        WorthServerOperationRequestDenialCode::MissingOperationFamily,
        diagnostics_profile,
        "operation request requires an operation family before planning",
    )
}

pub(super) fn missing_name(
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerOperationRequestDenial {
    WorthServerOperationRequestDenial::new(
        WorthServerOperationRequestDenialCode::MissingOperationName,
        diagnostics_profile,
        "operation request requires a declared operation name before planning",
    )
}

pub(super) fn admit_identifier(
    value: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    denial_code: WorthServerOperationRequestDenialCode,
    label: &str,
) -> Result<String, WorthServerOperationRequestDenial> {
    let admitted = WorthServerCanonicalFilename::admit(
        value,
        diagnostics_profile,
        crate::WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
    )
    .map_err(|denial| {
        WorthServerOperationRequestDenial::new(
            denial_code.clone(),
            diagnostics_profile,
            denial.detail(),
        )
    })?;
    if admitted.canonical().contains(' ') {
        return Err(WorthServerOperationRequestDenial::new(
            denial_code,
            diagnostics_profile,
            format!("{label} may not contain whitespace"),
        ));
    }
    Ok(admitted.canonical().to_string())
}

pub(super) fn admit_optional_token(
    value: Option<&str>,
    diagnostics_profile: DiagnosticRichnessProfile,
    denial_code: WorthServerOperationRequestDenialCode,
    label: &str,
) -> Result<Option<String>, WorthServerOperationRequestDenial> {
    value
        .map(|value| admit_token(value, diagnostics_profile, denial_code, label))
        .transpose()
}

fn admit_token(
    value: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    denial_code: WorthServerOperationRequestDenialCode,
    label: &str,
) -> Result<String, WorthServerOperationRequestDenial> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WorthServerOperationRequestDenial::new(
            denial_code,
            diagnostics_profile,
            format!("{label} may not be blank"),
        ));
    }
    if trimmed.chars().any(|ch| ch.is_control() || !ch.is_ascii()) {
        return Err(WorthServerOperationRequestDenial::new(
            denial_code,
            diagnostics_profile,
            format!("{label} must stay ASCII-printable"),
        ));
    }
    Ok(trimmed.to_string())
}

pub(super) fn validate_payload_envelope(
    payload_envelope: Option<&WorthServerOperationInputEnvelope>,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> Result<(), WorthServerOperationRequestDenial> {
    let Some(payload_envelope) = payload_envelope else {
        return Ok(());
    };
    if payload_envelope.payload_identity().trim().is_empty() {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidPayloadEnvelope,
            diagnostics_profile,
            "operation payload envelope must carry a non-blank payload identity",
        ));
    }
    if let Some(schema_identity) = payload_envelope.declared_schema_identity()
        && (schema_identity.trim().is_empty()
            || schema_identity
                .chars()
                .any(|ch| ch.is_control() || !ch.is_ascii()))
    {
        return Err(WorthServerOperationRequestDenial::new(
            WorthServerOperationRequestDenialCode::InvalidDeclaredSchemaIdentity,
            diagnostics_profile,
            "operation payload declared schema identity must stay ASCII-printable and non-blank",
        ));
    }
    Ok(())
}

pub(super) fn request_context_digest(
    resolved_request_context: &WorthServerResolvedRequestContext,
) -> String {
    let request_context = resolved_request_context.request_context();
    format!(
        "surface={:?};transport={:?};principal={};tenant={};workspace={};branch={};diagnostics={:?}",
        resolved_request_context.surface_family(),
        resolved_request_context.transport_class(),
        request_context.authenticated_principal().principal_id(),
        request_context.workspace_target().tenant_id(),
        request_context.workspace_target().workspace_id(),
        request_context.branch_target().branch_digest(),
        request_context.diagnostics_profile(),
    )
}
