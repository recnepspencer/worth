use std::sync::Arc;

use worth_foundational::expression_api::{
    ExpressionConsumption, ExpressionCost, ExpressionInputs, ExpressionProfile, ExpressionValue,
};

use super::operand_binding::{bind_operands, UiBoundOperands, UiExpressionInputs};
use super::operand_currentness::operands_are_current;
use super::records::UiExpressionRecords;
use super::{
    UiExpressionDenialReason, UiExpressionEvaluationRecord, UiExpressionOperandFact,
    UiExpressionOutcome, UiExpressionWorkCounters,
};
use crate::runtime::expression::{UiExpressionCatalog, UiExpressionSlot, UiInstalledExpression};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// The observation an evaluation is entitled to complete: the generation it
/// began in, the installed expression, and the exact operand facts it read.
/// It holds the installed expression itself, so evaluating it looks nothing
/// up.
pub(crate) struct UiExpressionEvaluationTicket {
    installed: Arc<UiInstalledExpression>,
    generation: WorthUiActiveApplicationGenerationIdentity,
    slot: UiExpressionSlot,
    operands: UiBoundOperands,
}

/// A finished evaluation that has not yet been admitted as a record. It keeps
/// the installed expression its ticket held, so admitting it looks nothing up.
pub(crate) struct UiExpressionCompletion {
    installed: Arc<UiInstalledExpression>,
    generation: WorthUiActiveApplicationGenerationIdentity,
    slot: UiExpressionSlot,
    operands: Box<[UiExpressionOperandFact]>,
    outcome: UiExpressionOutcome,
    consumed: Option<ExpressionConsumption>,
    cost: Option<ExpressionCost>,
}

/// How a completion met the generation fence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiExpressionCompletionReceipt {
    /// The completion was admitted. `changed` is true when the outcome, value
    /// or posture, differs from the retained one and its revision advanced.
    Applied { changed: bool },
    /// The completion began in another prepared generation of this session,
    /// reached an owner that has not followed the active generation, read an
    /// operand fact its owner no longer holds, or names a slot the retained
    /// records do not hold.
    Stale,
    /// The completion began in another active session.
    WrongWorld,
}

/// Reads every operand once and returns the ticket for `slot`, or `None` when
/// the catalog has no such expression.
pub(super) fn begin(
    catalog: &UiExpressionCatalog,
    records: &UiExpressionRecords,
    slot: UiExpressionSlot,
    inputs: &UiExpressionInputs<'_>,
    counters: &mut UiExpressionWorkCounters,
) -> Option<UiExpressionEvaluationTicket> {
    let installed = catalog.expression(slot)?;
    let operands = bind_operands(installed, records, inputs, counters);
    Some(UiExpressionEvaluationTicket {
        installed: Arc::clone(installed),
        generation: inputs.generation.clone(),
        slot,
        operands,
    })
}

/// Settles the expression from its operands. A non-current operand settles it
/// without a kernel run; otherwise the compiled program runs once.
pub(super) fn evaluate(
    ticket: UiExpressionEvaluationTicket,
    counters: &mut UiExpressionWorkCounters,
) -> UiExpressionCompletion {
    let UiExpressionEvaluationTicket {
        installed,
        generation,
        slot,
        operands,
    } = ticket;
    let UiBoundOperands {
        facts,
        values,
        settled,
    } = operands;
    let (outcome, consumed, cost) = match settled {
        Some(outcome) => {
            counters.settled_without_evaluation =
                counters.settled_without_evaluation.saturating_add(1);
            (outcome, None, None)
        }
        None => {
            counters.evaluations = counters.evaluations.saturating_add(1);
            run_kernel(&installed, values)
        }
    };
    UiExpressionCompletion {
        installed,
        generation,
        slot,
        operands: facts,
        outcome,
        consumed,
        cost,
    }
}

fn run_kernel(
    installed: &UiInstalledExpression,
    values: Vec<(usize, ExpressionValue)>,
) -> (
    UiExpressionOutcome,
    Option<ExpressionConsumption>,
    Option<ExpressionCost>,
) {
    let bound = values.into_iter().try_fold(
        ExpressionInputs::builder(installed.schema()),
        |builder, (index, value)| builder.bind(installed.operands()[index].name(), value),
    );
    let inputs = match bound {
        Ok(builder) => builder.build(),
        Err(denial) => {
            return (
                UiExpressionOutcome::Denied(UiExpressionDenialReason::Evaluation(denial)),
                None,
                None,
            )
        }
    };
    let evaluation = installed
        .program()
        .evaluate(&inputs, &ExpressionProfile::interactive());
    let outcome = match evaluation.result() {
        Ok(value) => UiExpressionOutcome::of_role(installed.role(), value),
        Err(denial) => {
            UiExpressionOutcome::Denied(UiExpressionDenialReason::Evaluation(denial.clone()))
        }
    };
    (
        outcome,
        Some(evaluation.consumption().clone()),
        Some(*evaluation.cost()),
    )
}

/// Admits a completion into the retained records if it began in the active
/// generation, the records follow that generation, and every operand fact it
/// read is still what its owner holds. A fenced completion increments its
/// counter and changes no record.
pub(super) fn complete(
    records: &mut UiExpressionRecords,
    completion: UiExpressionCompletion,
    inputs: &UiExpressionInputs<'_>,
    records_follow: bool,
    counters: &mut UiExpressionWorkCounters,
) -> UiExpressionCompletionReceipt {
    let generation = inputs.generation;
    if completion.generation.session_identity() != generation.session_identity() {
        return refuse(counters, UiExpressionCompletionReceipt::WrongWorld);
    }
    if !records_follow || completion.generation != *generation {
        return refuse(counters, UiExpressionCompletionReceipt::Stale);
    }
    let installed = &completion.installed;
    if !operands_are_current(installed, &completion.operands, records, inputs, counters) {
        return refuse(counters, UiExpressionCompletionReceipt::Stale);
    }
    let previous = records.get(completion.slot);
    let outcome = carry_last_current(
        completion.outcome,
        previous.map(|retained| &retained.outcome),
    );
    let (outcome_revision, changed) = match previous {
        None => (1, true),
        Some(retained) if retained.outcome != outcome => {
            (retained.outcome_revision.saturating_add(1), true)
        }
        Some(retained) => (retained.outcome_revision, false),
    };
    let record = UiExpressionEvaluationRecord {
        slot: completion.slot,
        identity: installed.identity().into(),
        outcome,
        outcome_revision,
        operands: completion.operands,
        consumed: completion.consumed,
        program_identity: installed.program_identity().clone(),
        generation: completion.generation,
        cost: completion.cost,
        span: installed.body_span().cloned(),
    };
    if records.admit(record).is_err() {
        return refuse(counters, UiExpressionCompletionReceipt::Stale);
    }
    if changed {
        counters.published_changes = counters.published_changes.saturating_add(1);
    } else {
        counters.suppressed_unchanged = counters.suppressed_unchanged.saturating_add(1);
    }
    UiExpressionCompletionReceipt::Applied { changed }
}

fn refuse(
    counters: &mut UiExpressionWorkCounters,
    receipt: UiExpressionCompletionReceipt,
) -> UiExpressionCompletionReceipt {
    counters.stale_completions = counters.stale_completions.saturating_add(1);
    receipt
}

/// A stale outcome keeps the last value that was current, as evidence.
fn carry_last_current(
    outcome: UiExpressionOutcome,
    previous: Option<&UiExpressionOutcome>,
) -> UiExpressionOutcome {
    match (outcome, previous) {
        (
            UiExpressionOutcome::Stale {
                reason,
                last_current: None,
            },
            Some(retained),
        ) => UiExpressionOutcome::Stale {
            reason,
            last_current: match retained {
                UiExpressionOutcome::Stale { last_current, .. } => last_current.clone(),
                current @ (UiExpressionOutcome::Condition(_)
                | UiExpressionOutcome::Value(_)
                | UiExpressionOutcome::Denied(_)
                | UiExpressionOutcome::Unavailable(_)) => current.current_value(),
            },
        },
        (outcome, _) => outcome,
    }
}

#[cfg(test)]
mod tests {
    use worth_ui_dsl::WorthUiExpressionRole;

    use super::{carry_last_current, UiExpressionDenialReason, UiExpressionOutcome};
    use crate::runtime::expression::{UiExpressionCurrentValue, UiExpressionStaleReason};

    fn stale() -> UiExpressionOutcome {
        UiExpressionOutcome::Stale {
            reason: UiExpressionStaleReason::Upstream {
                operand: "up".into(),
                identity: "ex.up".into(),
            },
            last_current: None,
        }
    }

    #[test]
    fn a_stale_outcome_keeps_the_value_that_was_current_before_it() {
        let carried = carry_last_current(stale(), Some(&UiExpressionOutcome::Condition(true)));

        assert_eq!(
            carried,
            UiExpressionOutcome::Stale {
                reason: UiExpressionStaleReason::Upstream {
                    operand: "up".into(),
                    identity: "ex.up".into(),
                },
                last_current: Some(UiExpressionCurrentValue::Condition(true)),
            }
        );
    }

    #[test]
    fn a_stale_outcome_with_no_prior_value_carries_none() {
        assert_eq!(carry_last_current(stale(), None), stale());
        assert_eq!(
            carry_last_current(
                stale(),
                Some(&UiExpressionOutcome::Denied(
                    UiExpressionDenialReason::RoleMismatch {
                        role: WorthUiExpressionRole::Condition
                    }
                ))
            ),
            stale(),
            "a denial held no value to keep"
        );
    }

    #[test]
    fn a_second_stale_outcome_keeps_the_first_last_current_value() {
        let first = carry_last_current(stale(), Some(&UiExpressionOutcome::Condition(false)));

        let second = carry_last_current(stale(), Some(&first));

        assert_eq!(second, first);
    }

    #[test]
    fn a_current_or_non_stale_outcome_is_never_rewritten() {
        let current = UiExpressionOutcome::Condition(true);
        assert_eq!(
            carry_last_current(
                current.clone(),
                Some(&UiExpressionOutcome::Condition(false))
            ),
            current
        );
    }
}
