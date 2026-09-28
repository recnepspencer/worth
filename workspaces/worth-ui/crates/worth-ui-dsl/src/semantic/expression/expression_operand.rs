use super::WorthUiExpressionOperandSource;

/// One named operand of an expression declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthUiExpressionOperand {
    name: String,
    source: WorthUiExpressionOperandSource,
}

impl WorthUiExpressionOperand {
    pub fn new(name: impl Into<String>, source: WorthUiExpressionOperandSource) -> Self {
        Self {
            name: name.into(),
            source,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &WorthUiExpressionOperandSource {
        &self.source
    }
}
