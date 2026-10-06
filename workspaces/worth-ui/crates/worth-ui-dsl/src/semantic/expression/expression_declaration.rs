use std::collections::BTreeSet;

use worth_foundational::expression_api::{ExpressionSchema, ExpressionType};

use super::{
    WorthUiExpressionBody, WorthUiExpressionDeclarationError,
    WorthUiExpressionDeclarationErrorKind as ErrorKind, WorthUiExpressionIntroducer,
    WorthUiExpressionOperand, WorthUiExpressionResultType, WorthUiExpressionRole,
};

/// A well-formed, not yet admitted expression declaration: identity, role,
/// named operands in name order, and the raw kernel expression text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiExpressionDeclaration {
    identity: String,
    role: WorthUiExpressionRole,
    operands: Vec<WorthUiExpressionOperand>,
    body: WorthUiExpressionBody,
}

impl WorthUiExpressionDeclaration {
    pub(crate) fn condition(
        identity: impl Into<String>,
        operands: Vec<WorthUiExpressionOperand>,
        source: impl Into<String>,
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        Self::build(
            identity.into(),
            WorthUiExpressionRole::Condition,
            operands,
            WorthUiExpressionBody::new(WorthUiExpressionIntroducer::When, source, 0),
        )
    }

    pub(crate) fn derived(
        identity: impl Into<String>,
        operands: Vec<WorthUiExpressionOperand>,
        result: WorthUiExpressionResultType,
        source: impl Into<String>,
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        Self::build(
            identity.into(),
            WorthUiExpressionRole::Derived(result),
            operands,
            WorthUiExpressionBody::new(WorthUiExpressionIntroducer::Value, source, 0),
        )
    }

    pub(super) fn build(
        identity: String,
        role: WorthUiExpressionRole,
        mut operands: Vec<WorthUiExpressionOperand>,
        body: WorthUiExpressionBody,
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        if identity.is_empty() {
            return Err(WorthUiExpressionDeclarationError::new(
                ErrorKind::EmptyIdentity,
                "expression declaration identity must not be empty",
            ));
        }
        operands.sort_by(|left, right| left.name().cmp(right.name()));
        let mut names = BTreeSet::new();
        for operand in &operands {
            validate_operand_name(operand.name())?;
            if !names.insert(operand.name()) {
                return Err(WorthUiExpressionDeclarationError::new(
                    ErrorKind::DuplicateOperand,
                    format!("operand `{}` is declared more than once", operand.name()),
                ));
            }
        }
        Ok(Self {
            identity,
            role,
            operands,
            body,
        })
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn role(&self) -> WorthUiExpressionRole {
        self.role
    }

    pub(crate) fn operands(&self) -> &[WorthUiExpressionOperand] {
        &self.operands
    }

    pub(crate) fn body(&self) -> &WorthUiExpressionBody {
        &self.body
    }
}

/// The kernel owns what a binding name is; asking it once keeps that rule in
/// one place.
fn validate_operand_name(name: &str) -> Result<(), WorthUiExpressionDeclarationError> {
    ExpressionSchema::builder()
        .operand(name, ExpressionType::Bool)
        .map(|_| ())
        .map_err(|_| {
            WorthUiExpressionDeclarationError::new(
                ErrorKind::InvalidOperandName,
                format!("`{name}` is not a valid expression operand name"),
            )
        })
}
