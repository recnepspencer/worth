use worth_foundational::expression_api::ExpressionType;

use crate::WorthUiExpressionOperandSource;

/// One operand of a sealed expression, with the kernel type its source
/// resolved to when the package compiled.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WorthUiSealedExpressionOperand {
    name: String,
    source: WorthUiExpressionOperandSource,
    expression_type: ExpressionType,
}

impl WorthUiSealedExpressionOperand {
    pub(super) fn new(
        name: String,
        source: WorthUiExpressionOperandSource,
        expression_type: ExpressionType,
    ) -> Self {
        Self {
            name,
            source,
            expression_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &WorthUiExpressionOperandSource {
        &self.source
    }

    pub fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }
}
