use crate::semantic::WorthUiExpressionDeclaration;
use crate::{
    WorthUiExpressionDeclarationError, WorthUiExpressionOperand, WorthUiExpressionResultType,
};

impl super::WorthUiRustAuthoredArtifactInputModule {
    /// Declares a `condition` block: a boolean expression over named
    /// operands, admitted by the shared kernel when the package compiles.
    pub fn try_with_condition(
        mut self,
        identity: impl Into<String>,
        operands: impl IntoIterator<Item = WorthUiExpressionOperand>,
        source: impl Into<String>,
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        let declaration = WorthUiExpressionDeclaration::condition(
            identity,
            operands.into_iter().collect(),
            source,
        )?;
        self.declarations
            .push(super::WorthUiRustAuthoredDeclaration::Condition {
                name_text: declaration.identity().to_owned(),
                body_atoms: declaration.body_atoms(),
            });
        Ok(self)
    }

    /// Declares a `derived` block: a typed value expression over named
    /// operands, admitted by the shared kernel when the package compiles.
    pub fn try_with_derived(
        mut self,
        identity: impl Into<String>,
        operands: impl IntoIterator<Item = WorthUiExpressionOperand>,
        result: WorthUiExpressionResultType,
        source: impl Into<String>,
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        let declaration = WorthUiExpressionDeclaration::derived(
            identity,
            operands.into_iter().collect(),
            result,
            source,
        )?;
        self.declarations
            .push(super::WorthUiRustAuthoredDeclaration::Derived {
                name_text: declaration.identity().to_owned(),
                body_atoms: declaration.body_atoms(),
            });
        Ok(self)
    }
}
