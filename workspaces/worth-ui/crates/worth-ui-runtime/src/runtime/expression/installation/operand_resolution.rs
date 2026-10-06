use std::collections::BTreeMap;

use worth_ui_dsl::{WorthUiExpressionOperandSource, WorthUiSealedExpressionOperand};
use worth_ui_query_binding::{
    UiProjectionInputSlot, UiProjectionNativeFamily, WorthUiQueryBindingPlan,
    WorthUiQueryViewIdentity,
};

use super::{UiExpressionCatalogPreparationDenial, UiExpressionSlot};
use crate::capability::UiIntentPayloadFieldKind;
use crate::declaration::{UiIntentApplicationFactPlan, UiIntentApplicationFactSlot};
use crate::runtime::WorthUiAuthoredProjectionRequirement;

/// Where an installed operand reads its value, resolved once at preparation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiResolvedExpressionOperand {
    QueryScalar {
        view: WorthUiQueryViewIdentity,
        slot: UiProjectionInputSlot,
    },
    Application {
        slot: UiIntentApplicationFactSlot,
        kind: UiIntentPayloadFieldKind,
        fact: Box<str>,
    },
    Expression {
        slot: UiExpressionSlot,
        identity: Box<str>,
    },
}

/// Resolves the operands that DSL admission already proved well typed. Only
/// the runtime-owned bindings are checked here: the fact plan, the query
/// binding plan, and the slots of the other installed expressions.
pub(super) struct UiExpressionOperandResolver<'a> {
    pub(super) projections: &'a [WorthUiAuthoredProjectionRequirement],
    pub(super) query: &'a WorthUiQueryBindingPlan,
    pub(super) application_facts: &'a UiIntentApplicationFactPlan,
    pub(super) expression_slots: &'a BTreeMap<&'a str, UiExpressionSlot>,
}

impl UiExpressionOperandResolver<'_> {
    pub(super) fn resolve(
        &self,
        expression: &str,
        operand: &WorthUiSealedExpressionOperand,
    ) -> Result<UiResolvedExpressionOperand, UiExpressionCatalogPreparationDenial> {
        match operand.source() {
            WorthUiExpressionOperandSource::QueryScalar { projection } => {
                self.query_scalar(expression, operand.name(), projection)
            }
            WorthUiExpressionOperandSource::ApplicationBoolean { fact } => self.application(
                expression,
                operand.name(),
                fact,
                UiIntentPayloadFieldKind::Boolean,
            ),
            WorthUiExpressionOperandSource::ApplicationUnsigned64 { fact } => self.application(
                expression,
                operand.name(),
                fact,
                UiIntentPayloadFieldKind::Unsigned64,
            ),
            WorthUiExpressionOperandSource::ApplicationText { fact } => self.application(
                expression,
                operand.name(),
                fact,
                UiIntentPayloadFieldKind::Text,
            ),
            WorthUiExpressionOperandSource::Condition { identity }
            | WorthUiExpressionOperandSource::Derived { identity } => self
                .expression_slots
                .get(identity.as_str())
                .copied()
                .map(|slot| UiResolvedExpressionOperand::Expression {
                    slot,
                    identity: identity.as_str().into(),
                })
                .ok_or_else(
                    || UiExpressionCatalogPreparationDenial::UnknownExpressionOperand {
                        expression: expression.into(),
                        operand: operand.name().into(),
                        identity: identity.as_str().into(),
                    },
                ),
        }
    }

    fn query_scalar(
        &self,
        expression: &str,
        operand: &str,
        projection: &str,
    ) -> Result<UiResolvedExpressionOperand, UiExpressionCatalogPreparationDenial> {
        let (requirement, scalar) = self
            .projections
            .iter()
            .find(|requirement| requirement.declaration_identity() == projection)
            .and_then(|requirement| Some((requirement, requirement.scalar_requirement()?)))
            .ok_or_else(
                || UiExpressionCatalogPreparationDenial::UnknownQueryScalar {
                    expression: expression.into(),
                    operand: operand.into(),
                    projection: projection.into(),
                },
            )?;
        match scalar.native_family() {
            UiProjectionNativeFamily::Text => {}
            UiProjectionNativeFamily::Boolean => {
                return Err(
                    UiExpressionCatalogPreparationDenial::UnsupportedQueryScalarType {
                        expression: expression.into(),
                        operand: operand.into(),
                        projection: projection.into(),
                    },
                );
            }
        }
        let view = requirement.view_identity().clone();
        let slot = self.query.projection_input_slot(&view).ok_or_else(|| {
            UiExpressionCatalogPreparationDenial::UnboundProjection {
                expression: expression.into(),
                operand: operand.into(),
                view: view.as_str().into(),
            }
        })?;
        Ok(UiResolvedExpressionOperand::QueryScalar { view, slot })
    }

    fn application(
        &self,
        expression: &str,
        operand: &str,
        fact: &str,
        expected: UiIntentPayloadFieldKind,
    ) -> Result<UiResolvedExpressionOperand, UiExpressionCatalogPreparationDenial> {
        let definition = self.application_facts.get(fact).ok_or_else(|| {
            UiExpressionCatalogPreparationDenial::UnknownApplicationFact {
                expression: expression.into(),
                operand: operand.into(),
                fact: fact.into(),
            }
        })?;
        if definition.kind() != expected {
            return Err(
                UiExpressionCatalogPreparationDenial::ApplicationFactKindMismatch {
                    expression: expression.into(),
                    operand: operand.into(),
                    fact: fact.into(),
                    expected,
                    observed: definition.kind(),
                },
            );
        }
        Ok(UiResolvedExpressionOperand::Application {
            slot: definition.slot(),
            kind: expected,
            fact: fact.into(),
        })
    }
}
