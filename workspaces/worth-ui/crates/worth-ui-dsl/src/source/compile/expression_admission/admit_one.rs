use super::super::WorthUiSealedExpression;
use super::body_admission::admit_body;
use super::canonical_form::canonical_form_diagnostics;
use super::operand_resolution::resolve_operands;
use super::operand_types::OperandScope;
use super::pending_expression::PendingExpression;
use crate::source::{
    WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic, WorthUiDslSourceSpan,
};

/// Admits one declaration through the shared kernel and seals the result:
/// operands first, then the body, then the canonical-form rules.
pub(super) fn admit_one(
    pending: &PendingExpression,
    scope: &OperandScope<'_>,
) -> Result<WorthUiSealedExpression, Vec<WorthUiDslCompileDiagnostic>> {
    let operands = resolve_operands(pending, scope)?;
    let body = admit_body(pending, &operands).map_err(|diagnostic| vec![diagnostic])?;
    let violations = canonical_form_diagnostics(pending, &operands, &body.admitted);
    if !violations.is_empty() {
        return Err(violations);
    }
    let declaration = &pending.declaration;
    Ok(WorthUiSealedExpression::new(
        declaration.identity().to_owned(),
        declaration.role(),
        operands.into_boxed_slice(),
        body.schema,
        body.encoded_draft,
        body.admitted,
        pending.provenance_ref,
        host_body_span(pending),
    ))
}

/// The body's byte range in the host file. Rust-authored declarations have
/// no host file.
fn host_body_span(pending: &PendingExpression) -> Option<WorthUiDslSourceSpan> {
    let body = pending.declaration.body();
    match &pending.provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span, ..
        } => Some(WorthUiDslSourceSpan::new(
            declaration_span.module_id().as_str(),
            body.body_start(),
            body.body_start() + body.source().len(),
        )),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration { .. } => None,
    }
}
