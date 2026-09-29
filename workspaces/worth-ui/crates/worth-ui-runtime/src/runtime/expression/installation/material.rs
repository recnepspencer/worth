use worth_ui_dsl::{WorthUiSealedExpression, WorthUiSealedSemanticPackage};

/// Installation input for the expression catalog: the package's sealed
/// expressions in identity order.
///
/// This is not handoff evidence. It rides beside the evidence so the runtime
/// installs each expression once per prepared generation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthUiAuthoredExpressionMaterial {
    pub(super) expressions: Box<[WorthUiSealedExpression]>,
}

impl WorthUiAuthoredExpressionMaterial {
    pub(crate) fn from_package(package: &WorthUiSealedSemanticPackage) -> Self {
        Self {
            expressions: package.expressions().cloned().collect(),
        }
    }

    pub(crate) fn expressions(&self) -> &[WorthUiSealedExpression] {
        &self.expressions
    }
}
