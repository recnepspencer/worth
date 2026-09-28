use worth_foundational::expression_api::{
    expressions, AdmittedExpression, ExpressionFunctionCatalog, ExpressionProfile,
    ExpressionSchema, ExpressionType,
};

use super::super::WorthUiSealedExpressionOperand;
use super::diagnostics::{declaration_diagnostic, kernel_denial_diagnostic};
use super::operand_types::result_expression_type;
use super::pending_expression::PendingExpression;
use crate::source::{WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode};
use crate::WorthUiExpressionRole;

/// The one resource profile every UI expression is admitted under.
const PROFILE: ExpressionProfile = ExpressionProfile::interactive();

/// What the kernel returns for an admitted body.
pub(super) struct AdmittedBody {
    pub(super) schema: ExpressionSchema,
    pub(super) encoded_draft: Vec<u8>,
    pub(super) admitted: AdmittedExpression,
}

/// Reads the body and admits it against the schema its operands declare and
/// the result type its role demands.
pub(super) fn admit_body(
    pending: &PendingExpression,
    operands: &[WorthUiSealedExpressionOperand],
) -> Result<AdmittedBody, WorthUiDslCompileDiagnostic> {
    let declaration = &pending.declaration;
    let body = declaration.body();
    let denied = |denial| {
        kernel_denial_diagnostic(
            &denial,
            declaration.identity(),
            &pending.provenance,
            body.body_start(),
            body.source().len(),
        )
    };
    let schema = build_schema(operands).map_err(|message| {
        declaration_diagnostic(
            WorthUiDslCompileDiagnosticCode::InvalidExpressionDeclaration,
            message,
            &pending.provenance,
        )
    })?;
    let catalog = ExpressionFunctionCatalog::builder(&schema, PROFILE).build();
    let expected = match declaration.role() {
        WorthUiExpressionRole::Condition => ExpressionType::Bool,
        WorthUiExpressionRole::Derived(result) => result_expression_type(result),
    };
    let draft = expressions()
        .reading_within(PROFILE)
        .parse(body.source())
        .map_err(denied)?;
    let admitted = draft
        .admit_as(&schema, &catalog, PROFILE, &expected)
        .map_err(denied)?;
    Ok(AdmittedBody {
        schema,
        encoded_draft: draft.encode(),
        admitted,
    })
}

fn build_schema(operands: &[WorthUiSealedExpressionOperand]) -> Result<ExpressionSchema, String> {
    let mut builder = ExpressionSchema::builder();
    for operand in operands {
        builder = builder
            .operand(operand.name(), operand.expression_type().clone())
            .map_err(|denial| {
                format!(
                    "operand `{}` was refused by the expression kernel: {:?}",
                    operand.name(),
                    denial.detail()
                )
            })?;
    }
    Ok(builder.build())
}
