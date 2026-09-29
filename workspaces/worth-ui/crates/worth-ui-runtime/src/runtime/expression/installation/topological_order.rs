use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{WorthUiExpressionOperandSource, WorthUiSealedExpression};

use super::UiExpressionCatalogPreparationDenial;

/// Kahn order over the expression-to-expression operand edges.
///
/// The result lists indexes into `expressions`. Ties resolve to the lowest
/// index, so the order is deterministic and follows identity order among
/// independent expressions.
pub(super) fn topological_order(
    expressions: &[WorthUiSealedExpression],
) -> Result<Vec<usize>, UiExpressionCatalogPreparationDenial> {
    let indexes: BTreeMap<&str, usize> = expressions
        .iter()
        .enumerate()
        .map(|(index, expression)| (expression.identity(), index))
        .collect();
    let mut unmet = vec![0usize; expressions.len()];
    let mut readers: Vec<Vec<usize>> = vec![Vec::new(); expressions.len()];
    for (reader, expression) in expressions.iter().enumerate() {
        let upstream: BTreeSet<usize> = expression
            .operands()
            .iter()
            .filter_map(|operand| match operand.source() {
                WorthUiExpressionOperandSource::Condition { identity }
                | WorthUiExpressionOperandSource::Derived { identity } => Some((operand, identity)),
                WorthUiExpressionOperandSource::QueryScalar { .. }
                | WorthUiExpressionOperandSource::ApplicationBoolean { .. }
                | WorthUiExpressionOperandSource::ApplicationUnsigned64 { .. }
                | WorthUiExpressionOperandSource::ApplicationText { .. } => None,
            })
            .map(|(operand, identity)| {
                indexes.get(identity.as_str()).copied().ok_or_else(|| {
                    UiExpressionCatalogPreparationDenial::UnknownExpressionOperand {
                        expression: expression.identity().into(),
                        operand: operand.name().into(),
                        identity: identity.as_str().into(),
                    }
                })
            })
            .collect::<Result<_, _>>()?;
        unmet[reader] = upstream.len();
        for source in upstream {
            readers[source].push(reader);
        }
    }
    let mut ready: BTreeSet<usize> = (0..expressions.len())
        .filter(|index| unmet[*index] == 0)
        .collect();
    let mut order = Vec::with_capacity(expressions.len());
    while let Some(next) = ready.pop_first() {
        order.push(next);
        for &reader in &readers[next] {
            unmet[reader] -= 1;
            if unmet[reader] == 0 {
                ready.insert(reader);
            }
        }
    }
    if let Some(stuck) = (0..expressions.len()).find(|index| unmet[*index] > 0) {
        return Err(UiExpressionCatalogPreparationDenial::ExpressionCycle {
            expression: expressions[stuck].identity().into(),
        });
    }
    Ok(order)
}
