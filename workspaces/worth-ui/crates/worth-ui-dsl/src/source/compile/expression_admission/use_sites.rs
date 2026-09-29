//! Consumer use sites of admitted expressions. A declaration that names an
//! expression must name one the package admitted, in the role the use site
//! reads. An expression refused at admission already carries its own
//! diagnostic, so its use sites add none.
//!
//! The package knows each expression's role and result type but not the
//! field kinds of a payload schema, so a payload `derived` source is checked
//! here only for being a payload value at all; whether its result type fits
//! the field is the runtime intent catalog's check.

use super::super::{WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};
use super::diagnostics::declaration_diagnostic;
use crate::source::WorthUiDslCompileDiagnosticCode;
use crate::{WorthUiExpressionResultType, WorthUiExpressionRole, WorthUiIntentPayloadSource};

/// What a use site reads from the expression it names.
#[derive(Clone, Copy)]
enum UseSiteRead {
    /// The one Boolean form: operability axes and Boolean payload fields.
    Condition,
    /// A derived value a payload field can carry: any result type but an
    /// appearance token.
    PayloadValue,
}

impl WorthUiSemanticPackageSealingState {
    /// Checks every intent operability axis and payload field that reads an
    /// expression against the admitted expression table.
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
                let consumer = artifact.declaration().key().as_str();
                let operability = intent.operability();
                let axes = [
                    ("mutability", operability.mutability().condition_identity()),
                    ("readiness", operability.readiness().condition_identity()),
                    ("policy", operability.policy().condition_identity()),
                ]
                .into_iter()
                .filter_map(|(axis, condition)| {
                    let site = format!("intent `{consumer}` operability {axis}");
                    Some((site, UseSiteRead::Condition, condition?))
                });
                let payload = intent.payload_sources().iter().filter_map(|source| {
                    let site = format!("intent `{consumer}` payload field `{}`", source.field());
                    match source.source() {
                        WorthUiIntentPayloadSource::Derived { expression } => {
                            Some((site, UseSiteRead::PayloadValue, &**expression))
                        }
                        WorthUiIntentPayloadSource::Condition { expression } => {
                            Some((site, UseSiteRead::Condition, &**expression))
                        }
                        _ => None,
                    }
                });
                for (site, read, identity) in axes.chain(payload) {
                    if let Some((code, message)) = self.use_denial(&site, read, identity) {
                        let provenance = &self.provenance_table[artifact.provenance_ref().0];
                        found.push(declaration_diagnostic(code, message, provenance));
                    }
                }
            }
        }
        self.diagnostics.append(&mut found);
    }

    fn use_denial(
        &self,
        site: &str,
        read: UseSiteRead,
        identity: &str,
    ) -> Option<(WorthUiDslCompileDiagnosticCode, String)> {
        let mismatch = |detail: &str| {
            Some((
                WorthUiDslCompileDiagnosticCode::ExpressionUseSiteRoleMismatch,
                format!("{site} reads `{identity}`, {detail}"),
            ))
        };
        match (self.expressions.get(identity), read) {
            (Some(expression), UseSiteRead::Condition) => match expression.role() {
                WorthUiExpressionRole::Condition => None,
                WorthUiExpressionRole::Derived(_) => {
                    mismatch("which is a derived declaration, not a condition")
                }
            },
            (Some(expression), UseSiteRead::PayloadValue) => match expression.role() {
                WorthUiExpressionRole::Derived(WorthUiExpressionResultType::Token) => {
                    mismatch("a derived token, which no payload field carries")
                }
                WorthUiExpressionRole::Derived(_) => None,
                WorthUiExpressionRole::Condition => mismatch(
                    "which is a condition, not a derived value; a Boolean field reads it with `condition`",
                ),
            },
            (None, _) if self.refused_expressions.contains(identity) => None,
            (None, read) => {
                let noun = match read {
                    UseSiteRead::Condition => "condition",
                    UseSiteRead::PayloadValue => "derived value",
                };
                Some((
                    WorthUiDslCompileDiagnosticCode::UnknownExpressionUseSite,
                    format!("{site} reads {noun} `{identity}`, which is not declared"),
                ))
            }
        }
    }
}
