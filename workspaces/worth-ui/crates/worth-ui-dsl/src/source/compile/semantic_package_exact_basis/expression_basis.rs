use worth_foundational::expression_api::ExpressionProgramIdentity;

use super::Fingerprint;
use crate::source::WorthUiSealedExpression;

/// The exact meaning of one sealed expression: its identity, role, operands
/// with their resolved kernel types, and the complete kernel program identity.
/// Source text, spans, and formatting never enter it; the kernel identity
/// already excludes them. Equality compares the full identity, so an equal
/// digest alone never makes two expressions the same basis.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct WorthUiExpressionExactBasis {
    identity: String,
    role: String,
    operands: Box<[WorthUiExpressionOperandExactBasis]>,
    program_identity: ExpressionProgramIdentity,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiExpressionOperandExactBasis {
    name: String,
    source_kind: &'static str,
    source_reference: String,
    expression_type: String,
}

impl WorthUiExpressionExactBasis {
    pub(super) fn from_expression(expression: &WorthUiSealedExpression) -> Self {
        Self {
            identity: expression.identity().to_owned(),
            role: expression.role().canonical_token(),
            operands: expression
                .operands()
                .iter()
                .map(|operand| WorthUiExpressionOperandExactBasis {
                    name: operand.name().to_owned(),
                    source_kind: operand.source().clause_keyword(),
                    source_reference: operand.source().reference().to_owned(),
                    expression_type: operand.expression_type().to_string(),
                })
                .collect(),
            program_identity: expression.program_identity().clone(),
        }
    }

    pub(super) fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text("expression");
        fingerprint.fold_text(&self.identity);
        fingerprint.fold_text(&self.role);
        fingerprint.fold_usize(self.operands.len());
        for operand in &self.operands {
            fingerprint.fold_text(&operand.name);
            fingerprint.fold_text(operand.source_kind);
            fingerprint.fold_text(&operand.source_reference);
            fingerprint.fold_text(&operand.expression_type);
        }
        fingerprint.fold_bytes(self.program_identity.digest().value().bytes());
    }
}
