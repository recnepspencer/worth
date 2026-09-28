use std::collections::BTreeSet;

use worth_foundational::expression_api::AdmittedExpression;

use super::super::WorthUiSealedExpressionOperand;
use super::diagnostics::body_diagnostic;
use super::pending_expression::PendingExpression;
use crate::source::{
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass,
};

/// The canonical-form rules that sit on top of kernel admission: a body may
/// not bind `let`, and every declared operand must be read. Every violation
/// is reported.
pub(super) fn canonical_form_diagnostics(
    pending: &PendingExpression,
    operands: &[WorthUiSealedExpressionOperand],
    admitted: &AdmittedExpression,
) -> Vec<WorthUiDslCompileDiagnostic> {
    let identity = pending.declaration.identity();
    let body = pending.declaration.body();
    let whole_body = |code, message: String| {
        body_diagnostic(
            code,
            WorthUiDslCompileStopClass::LanguageLegality,
            message,
            &pending.provenance,
            body.body_start(),
            0..body.source().len(),
        )
    };
    let mut diagnostics = Vec::new();
    if admitted.binds_let() {
        diagnostics.push(whole_body(
            WorthUiDslCompileDiagnosticCode::ExpressionAdmissionDenied,
            format!(
                "expression `{identity}` uses a `let` binding; declare a named `derived` \
                 declaration and read it as an operand instead"
            ),
        ));
    }
    let read: BTreeSet<&str> = admitted.slots().map(|(name, _)| name).collect();
    for operand in operands
        .iter()
        .filter(|operand| !read.contains(operand.name()))
    {
        diagnostics.push(whole_body(
            WorthUiDslCompileDiagnosticCode::UnusedExpressionOperand,
            format!(
                "operand `{}` of expression `{identity}` is declared but never read",
                operand.name()
            ),
        ));
    }
    diagnostics
}
