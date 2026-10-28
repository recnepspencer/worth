use std::collections::BTreeSet;

use crate::{
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
};

use super::super::{
    validate_canonical_filename, validate_manifest_metadata_normalization,
    validate_operation_name_binding,
};
use super::request::{WorthServerMultipartUpload, WorthServerUploadExpectation};

const MAX_DECLARED_UPLOAD_PART_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) fn validate_upload_admission(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    upload: &WorthServerMultipartUpload,
    operation_name: &str,
) -> Result<(), WorthServerQueryHandoffDenial> {
    validate_upload_route_family(prepared_request)?;
    validate_upload_request_contract(prepared_request, upload.expectation())?;
    validate_operation_name_binding(
        prepared_request.request_contract(),
        operation_name,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        prepared_request
            .admission()
            .request_context()
            .diagnostics_profile(),
    )?;
    validate_upload_shape(prepared_request, upload, operation_name)?;
    Ok(())
}

fn validate_upload_route_family(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
) -> Result<(), WorthServerQueryHandoffDenial> {
    if prepared_request.request_contract().route_family()
        != WorthServerCompatHttpRouteFamily::Upload
    {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            prepared_request
                .admission()
                .request_context()
                .diagnostics_profile(),
            "compatibility upload execution requires the upload route family",
        ));
    }
    Ok(())
}

fn validate_upload_request_contract(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    expectation: WorthServerUploadExpectation,
) -> Result<(), WorthServerQueryHandoffDenial> {
    let diagnostics_profile = prepared_request
        .admission()
        .request_context()
        .diagnostics_profile();
    let request_contract = prepared_request.request_contract();
    if !request_contract.body_present() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility multipart upload requires a request body before admission can continue",
        ));
    }
    let Some(content_type) = request_contract.body_content_type() else {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility multipart upload requires an explicit multipart/form-data content type",
        ));
    };
    if !content_type.starts_with("multipart/form-data") {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            format!(
                "compatibility upload content type `{content_type}` is not admitted; expected multipart/form-data"
            ),
        ));
    }
    let expect_values = request_contract.canonical_headers().values("expect");
    if let Some(values) = expect_values
        && (values.len() != 1 || values[0] != "100-continue")
    {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload admits only `Expect: 100-continue` as an expectation header",
        ));
    }
    if expectation.requires_early_admission()
        && !matches!(expect_values, Some(values) if values == ["100-continue"])
    {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload expectation requires `Expect: 100-continue` so the server can deny before bulk body transfer begins",
        ));
    }
    Ok(())
}

fn validate_upload_shape(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    upload: &WorthServerMultipartUpload,
    operation_name: &str,
) -> Result<(), WorthServerQueryHandoffDenial> {
    let diagnostics_profile = prepared_request
        .admission()
        .request_context()
        .diagnostics_profile();
    if operation_name.trim().is_empty() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload operation name may not be blank",
        ));
    }
    validate_canonical_filename(
        operation_name,
        diagnostics_profile,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
    )?;
    let metadata_body = upload.manifest().metadata_body();
    if !metadata_body.is_object() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload manifest metadata must be a JSON object carrying the canonical mutation body",
        ));
    }
    validate_manifest_metadata_normalization(metadata_body, diagnostics_profile)?;
    let declared_parts = normalized_part_names(
        upload.manifest().declared_file_parts(),
        diagnostics_profile,
        "manifest file part",
    )?;
    if declared_parts.is_empty() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload manifest must declare at least one file part",
        ));
    }
    let observed_parts = normalized_upload_parts(upload, diagnostics_profile)?;
    if observed_parts.is_empty() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload must supply at least one declared file part",
        ));
    }
    let observed_names = observed_parts
        .iter()
        .map(|(name, _, _)| name.clone())
        .collect::<BTreeSet<_>>();
    let declared_names = declared_parts.into_iter().collect::<BTreeSet<_>>();
    if declared_names != observed_names {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            "compatibility upload part graph did not match the declared manifest file-part set",
        ));
    }
    Ok(())
}

fn normalized_part_names(
    part_names: &[String],
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
    label: &str,
) -> Result<Vec<String>, WorthServerQueryHandoffDenial> {
    let mut normalized = Vec::with_capacity(part_names.len());
    let mut seen = BTreeSet::new();
    for value in part_names {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                format!("compatibility upload {label} names may not be blank"),
            ));
        }
        if !seen.insert(trimmed.to_string()) {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                format!("compatibility upload {label} names may not repeat"),
            ));
        }
        normalized.push(trimmed.to_string());
    }
    Ok(normalized)
}

fn normalized_upload_parts(
    upload: &WorthServerMultipartUpload,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<Vec<(String, String, u64)>, WorthServerQueryHandoffDenial> {
    let mut normalized = Vec::with_capacity(upload.parts().len());
    let mut seen = BTreeSet::new();
    for part in upload.parts() {
        let name = part.name().trim();
        if name.is_empty() {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                "compatibility upload part names may not be blank",
            ));
        }
        if !seen.insert(name.to_string()) {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                format!("compatibility upload part `{name}` was supplied more than once"),
            ));
        }
        let content_type = part.content_type().trim().to_ascii_lowercase();
        if content_type.is_empty() {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                format!("compatibility upload part `{name}` requires an explicit content type"),
            ));
        }
        if part.declared_length() > MAX_DECLARED_UPLOAD_PART_BYTES {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
                diagnostics_profile,
                format!(
                    "compatibility upload part `{name}` declared `{}` bytes, which exceeds the phase-five early-admission cap `{MAX_DECLARED_UPLOAD_PART_BYTES}`",
                    part.declared_length()
                ),
            ));
        }
        normalized.push((name.to_string(), content_type, part.declared_length()));
    }
    Ok(normalized)
}
