use super::operand_binding::UiExpressionInputs;
use super::records::UiExpressionRecords;
use super::{UiExpressionOperandFact, UiExpressionWorkCounters};
use crate::runtime::expression::{UiInstalledExpression, UiResolvedExpressionOperand};

/// Whether every fact an evaluation read is still exactly what its owner
/// holds now. A completion whose operands moved on is stale, whatever its
/// generation. Each operand re-proven is one owner read and counts as one
/// operand probe; the first operand that moved on ends the proof.
pub(super) fn operands_are_current(
    installed: &UiInstalledExpression,
    read: &[UiExpressionOperandFact],
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
    counters: &mut UiExpressionWorkCounters,
) -> bool {
    installed.operands().len() == read.len()
        && installed
            .operands()
            .iter()
            .zip(read)
            .all(|(operand, fact)| {
                counters.operand_probes = counters.operand_probes.saturating_add(1);
                is_current(operand.source(), fact, records, inputs)
            })
}

fn is_current(
    source: &UiResolvedExpressionOperand,
    read: &UiExpressionOperandFact,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
) -> bool {
    match source {
        UiResolvedExpressionOperand::QueryScalar { slot, .. } => {
            let observed = inputs.mounted.current_projection_input(*slot);
            match read {
                UiExpressionOperandFact::Query(reference) => observed.as_ref() == Some(reference),
                UiExpressionOperandFact::Absent { .. } => observed.is_none(),
                UiExpressionOperandFact::Application(_)
                | UiExpressionOperandFact::Expression { .. } => false,
            }
        }
        UiResolvedExpressionOperand::Application { slot, .. } => match read {
            UiExpressionOperandFact::Application(reference) => inputs
                .facts
                .is_current_reference(reference, inputs.generation),
            UiExpressionOperandFact::Absent { .. } => inputs
                .facts
                .input_reference(*slot, inputs.generation)
                .is_none(),
            UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Expression { .. } => false,
        },
        UiResolvedExpressionOperand::Expression { slot, .. } => match read {
            UiExpressionOperandFact::Expression {
                slot: read_slot,
                outcome_revision,
            } => {
                read_slot == slot
                    && records
                        .get(*slot)
                        .is_some_and(|record| record.outcome_revision() == *outcome_revision)
            }
            UiExpressionOperandFact::Absent { .. } => records.get(*slot).is_none(),
            UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Application(_) => false,
        },
    }
}
