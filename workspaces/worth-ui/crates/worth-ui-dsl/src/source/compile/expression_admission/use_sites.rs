//! Consumer use sites of admitted expressions. A declaration that names an
//! expression must name one the package admitted, in the role the use site
//! reads. An expression refused at admission already carries its own
//! diagnostic, so its use sites add none.

use super::super::{WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};
use super::diagnostics::declaration_diagnostic;
use crate::source::WorthUiDslCompileDiagnosticCode;
use crate::WorthUiExpressionRole;

impl WorthUiSemanticPackageSealingState {
    /// Checks every intent operability axis that reads a condition against
    /// the admitted expression table.
    pub(in super::super) fn validate_expression_use_sites(&mut self) {
        let mut found = Vec::new();
        for module in self.modules.values() {
            for declaration in module.declarations() {
                let WorthUiSemanticDeclaration::SemanticArtifact(artifact) = declaration else {
                    continue;
                };
                let Some(intent) = artifact.declaration().intent_declaration() else {
                    continue;
                };
                let operability = intent.operability();
                let uses = [
                    ("mutability", operability.mutability().condition_identity()),
                    ("readiness", operability.readiness().condition_identity()),
                    ("policy", operability.policy().condition_identity()),
                ];
                let consumer = artifact.declaration().key().as_str();
                for (axis, condition) in uses {
                    let Some(condition) = condition else {
                        continue;
                    };
                    if let Some((code, message)) =
                        self.condition_use_denial(consumer, axis, condition)
                    {
                        let provenance = &self.provenance_table[artifact.provenance_ref().0];
                        found.push(declaration_diagnostic(code, message, provenance));
                    }
                }
            }
        }
        self.diagnostics.append(&mut found);
    }

    fn condition_use_denial(
        &self,
        consumer: &str,
        axis: &str,
        condition: &str,
    ) -> Option<(WorthUiDslCompileDiagnosticCode, String)> {
        match self.expressions.get(condition) {
            Some(expression) => match expression.role() {
                WorthUiExpressionRole::Condition => None,
                WorthUiExpressionRole::Derived(_) => Some((
                    WorthUiDslCompileDiagnosticCode::ExpressionUseSiteRoleMismatch,
                    format!(
                        "intent `{consumer}` operability {axis} reads `{condition}`, which is a derived declaration, not a condition"
                    ),
                )),
            },
            None if self.refused_expressions.contains(condition) => None,
            None => Some((
                WorthUiDslCompileDiagnosticCode::UnknownExpressionUseSite,
                format!(
                    "intent `{consumer}` operability {axis} reads condition `{condition}`, which is not declared"
                ),
            )),
        }
    }
}
