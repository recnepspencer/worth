use std::ops::Range;

use worth_foundational::expression_api::{
    ExpressionDenial, ExpressionDenialFamily, ExpressionOccurrence,
};

use super::super::sealing;
use crate::source::{
    WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode,
    WorthUiDslCompileStopClass, WorthUiDslSourceSpan,
};
use crate::WorthUiExpressionDeclarationErrorKind;

/// A whole-declaration diagnostic in the language-legality stop class.
pub(super) fn declaration_diagnostic(
    code: WorthUiDslCompileDiagnosticCode,
    message: impl Into<String>,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = sealing::diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        code,
        WorthUiDslCompileStopClass::LanguageLegality,
        message,
        Some(module_id),
        span,
    )
}

pub(super) fn malformed_declaration_diagnostic(
    error: &crate::WorthUiExpressionDeclarationError,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let code = match error.kind() {
        WorthUiExpressionDeclarationErrorKind::UnknownOperandSource => {
            WorthUiDslCompileDiagnosticCode::UnknownExpressionOperandSource
        }
        _ => WorthUiDslCompileDiagnosticCode::InvalidExpressionDeclaration,
    };
    declaration_diagnostic(code, error.detail(), provenance)
}

/// A diagnostic at `body_range` of the expression text, given as byte
/// offsets from the start of the body. Rust-authored declarations have no
/// host file, so they carry the module path and no span.
pub(super) fn body_diagnostic(
    code: WorthUiDslCompileDiagnosticCode,
    stop_class: WorthUiDslCompileStopClass,
    message: impl Into<String>,
    provenance: &WorthUiArtifactInputProvenance,
    body_start: usize,
    body_range: Range<usize>,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = match provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span, ..
        } => (
            declaration_span.module_id().as_str().to_owned(),
            Some(WorthUiDslSourceSpan::new(
                declaration_span.module_id().as_str(),
                body_start + body_range.start,
                body_start + body_range.end,
            )),
        ),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
            authored_module_path,
            ..
        } => (authored_module_path.clone(), None),
    };
    WorthUiDslCompileDiagnostic::new(code, stop_class, message, Some(module_id), span)
}

/// Maps a kernel denial to a compile diagnostic. Syntax denials keep the
/// syntax stop class; every other family is an admission denial that carries
/// the family and detail. A denial without a source span falls back to the
/// whole body.
pub(super) fn kernel_denial_diagnostic(
    denial: &ExpressionDenial,
    identity: &str,
    provenance: &WorthUiArtifactInputProvenance,
    body_start: usize,
    body_length: usize,
) -> WorthUiDslCompileDiagnostic {
    let (code, stop_class) = match denial.family() {
        ExpressionDenialFamily::Syntax => (
            WorthUiDslCompileDiagnosticCode::ExpressionSyntax,
            WorthUiDslCompileStopClass::LanguageSyntax,
        ),
        _ => (
            WorthUiDslCompileDiagnosticCode::ExpressionAdmissionDenied,
            WorthUiDslCompileStopClass::LanguageLegality,
        ),
    };
    let range = match denial.occurrence() {
        Some(ExpressionOccurrence::Source(span)) => span.start() as usize..span.end() as usize,
        Some(ExpressionOccurrence::Node(_)) | None => 0..body_length,
    };
    body_diagnostic(
        code,
        stop_class,
        format!(
            "expression `{identity}` denied ({:?}): {:?}",
            denial.family(),
            denial.detail()
        ),
        provenance,
        body_start,
        range,
    )
}
