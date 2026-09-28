use std::collections::BTreeMap;

use worth_foundational::expression_api::{ExpressionType, IntegerType};

use crate::source::{
    WorthUiDslCompileDiagnosticCode, WorthUiProjectionNativeFamily, WorthUiProjectionShape,
};
use crate::{
    WorthUiExpressionOperand, WorthUiExpressionOperandSource, WorthUiExpressionResultType,
    WorthUiExpressionRole,
};

/// Why an operand source has no kernel type.
pub(super) struct OperandDenial {
    pub(super) code: WorthUiDslCompileDiagnosticCode,
    pub(super) message: String,
}

/// The package facts an operand source resolves against.
pub(super) struct OperandScope<'package> {
    pub(super) projections:
        BTreeMap<&'package str, (WorthUiProjectionShape, WorthUiProjectionNativeFamily)>,
    pub(super) roles: BTreeMap<&'package str, WorthUiExpressionRole>,
}

pub(super) fn result_expression_type(result: WorthUiExpressionResultType) -> ExpressionType {
    match result {
        WorthUiExpressionResultType::Text | WorthUiExpressionResultType::Token => {
            ExpressionType::String
        }
        WorthUiExpressionResultType::Integer => ExpressionType::INT64,
        WorthUiExpressionResultType::Decimal => ExpressionType::Decimal,
    }
}

impl OperandScope<'_> {
    pub(super) fn resolve(
        &self,
        operand: &WorthUiExpressionOperand,
    ) -> Result<ExpressionType, OperandDenial> {
        let name = operand.name();
        match operand.source() {
            WorthUiExpressionOperandSource::ApplicationBoolean { .. } => Ok(ExpressionType::Bool),
            WorthUiExpressionOperandSource::ApplicationUnsigned64 { .. } => {
                Ok(ExpressionType::Integer(IntegerType::UInt64))
            }
            WorthUiExpressionOperandSource::ApplicationText { .. } => Ok(ExpressionType::String),
            WorthUiExpressionOperandSource::QueryScalar { projection } => {
                self.resolve_projection(name, projection)
            }
            WorthUiExpressionOperandSource::Condition { identity } => {
                match self.roles.get(identity.as_str()) {
                    None => Err(unknown(name, "condition", identity)),
                    Some(WorthUiExpressionRole::Condition) => Ok(ExpressionType::Bool),
                    Some(WorthUiExpressionRole::Derived(_)) => Err(invalid(format!(
                        "operand `{name}` reads `{identity}`, which is a derived declaration, \
                         not a condition"
                    ))),
                }
            }
            WorthUiExpressionOperandSource::Derived { identity } => {
                match self.roles.get(identity.as_str()) {
                    None => Err(unknown(name, "derived declaration", identity)),
                    Some(WorthUiExpressionRole::Derived(result)) => {
                        Ok(result_expression_type(*result))
                    }
                    Some(WorthUiExpressionRole::Condition) => Err(invalid(format!(
                        "operand `{name}` reads `{identity}`, which is a condition, not a \
                         derived declaration"
                    ))),
                }
            }
        }
    }

    fn resolve_projection(
        &self,
        name: &str,
        projection: &str,
    ) -> Result<ExpressionType, OperandDenial> {
        match self.projections.get(projection) {
            None => Err(unknown(name, "scalar projection", projection)),
            Some((WorthUiProjectionShape::Collection, _)) => Err(invalid(format!(
                "operand `{name}` reads projection `{projection}`, which is a collection; \
                 an expression operand needs a scalar projection"
            ))),
            Some((WorthUiProjectionShape::Scalar, WorthUiProjectionNativeFamily::Text)) => {
                Ok(ExpressionType::String)
            }
            Some((WorthUiProjectionShape::Scalar, WorthUiProjectionNativeFamily::Boolean)) => {
                Ok(ExpressionType::Bool)
            }
        }
    }
}

fn unknown(operand: &str, kind: &str, identity: &str) -> OperandDenial {
    OperandDenial {
        code: WorthUiDslCompileDiagnosticCode::UnknownExpressionOperandSource,
        message: format!("operand `{operand}` reads unknown {kind} `{identity}`"),
    }
}

fn invalid(message: String) -> OperandDenial {
    OperandDenial {
        code: WorthUiDslCompileDiagnosticCode::InvalidExpressionDeclaration,
        message,
    }
}
