use super::super::WorthUiSealedExpressionOperand;
use super::diagnostics::declaration_diagnostic;
use super::operand_types::OperandScope;
use super::pending_expression::PendingExpression;
use crate::source::WorthUiDslCompileDiagnostic;

/// Resolves every operand of `pending` to its kernel type. One run reports
/// every unresolved operand rather than stopping at the first.
pub(super) fn resolve_operands(
    pending: &PendingExpression,
    scope: &OperandScope<'_>,
) -> Result<Vec<WorthUiSealedExpressionOperand>, Vec<WorthUiDslCompileDiagnostic>> {
    let mut operands = Vec::new();
    let mut diagnostics = Vec::new();
    for operand in pending.declaration.operands() {
        match scope.resolve(operand) {
            Ok(expression_type) => operands.push(WorthUiSealedExpressionOperand::new(
                operand.name().to_owned(),
                operand.source().clone(),
                expression_type,
            )),
            Err(denial) => diagnostics.push(declaration_diagnostic(
                denial.code,
                denial.message,
                &pending.provenance,
            )),
        }
    }
    if diagnostics.is_empty() {
        Ok(operands)
    } else {
        Err(diagnostics)
    }
}
