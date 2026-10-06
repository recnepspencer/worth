use std::sync::Arc;

use worth_foundational::expression_api::ExpressionValue;
use worth_ui_query_binding::{
    UiProjectionFactStopKind, UiProjectionInputFactReference, UiProjectionInputPosture,
    UiProjectionInputSlot, WorthUiQueryViewIdentity,
};

use super::records::UiExpressionRecords;
use super::{
    UiExpressionDenialReason, UiExpressionOperandFact, UiExpressionOutcome,
    UiExpressionStaleReason, UiExpressionStopKind, UiExpressionUnavailableReason,
    UiExpressionWorkCounters,
};
use crate::capability::UiIntentPayloadFieldKind;
use crate::declaration::UiIntentApplicationFactSlot;
use crate::runtime::expression::{
    UiExpressionSlot, UiInstalledExpression, UiResolvedExpressionOperand,
};
use crate::runtime::intent::{UiIntentApplicationFactState, UiIntentApplicationInputReference};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

#[cfg(test)]
mod tests;

/// The owners an expression reads its operands from.
pub(crate) struct UiExpressionInputs<'state> {
    pub(crate) generation: &'state WorthUiActiveApplicationGenerationIdentity,
    pub(crate) mounted: &'state crate::mounting::WorthUiMountedSessionState,
    pub(crate) facts: &'state UiIntentApplicationFactState,
}

/// One expression's operands, observed once.
pub(super) struct UiBoundOperands {
    pub(super) facts: Box<[UiExpressionOperandFact]>,
    /// Current operand values by index into the installed operands.
    pub(super) values: Vec<(usize, ExpressionValue)>,
    /// The outcome the most severe non-current operand settles the expression
    /// with. Denied outranks unavailable, which outranks stale; name order
    /// only breaks a tie.
    pub(super) settled: Option<UiExpressionOutcome>,
}

enum Observation {
    Bound(ExpressionValue),
    NonCurrent(UiExpressionOutcome),
}

pub(super) fn bind_operands(
    installed: &UiInstalledExpression,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
    counters: &mut UiExpressionWorkCounters,
) -> UiBoundOperands {
    let mut facts = Vec::with_capacity(installed.operands().len());
    let mut values = Vec::new();
    let mut settled = None;
    for (index, operand) in installed.operands().iter().enumerate() {
        counters.operand_probes = counters.operand_probes.saturating_add(1);
        let name = operand.name();
        let (fact, observation) = match operand.source() {
            UiResolvedExpressionOperand::QueryScalar { view, slot } => {
                observe_query(name, view, *slot, inputs)
            }
            UiResolvedExpressionOperand::Application { slot, kind, fact } => {
                observe_application(name, fact, *slot, *kind, inputs)
            }
            UiResolvedExpressionOperand::Expression { slot, identity } => {
                observe_upstream(name, identity, *slot, records)
            }
        };
        facts.push(fact);
        match observation {
            Observation::Bound(value) => values.push((index, value)),
            Observation::NonCurrent(outcome) => withhold(&mut settled, outcome),
        }
    }
    UiBoundOperands {
        facts: facts.into_boxed_slice(),
        values,
        settled,
    }
}

/// Keeps the more severe of the held and the new non-current outcome. The
/// held one wins a tie, so name order breaks ties only.
fn withhold(settled: &mut Option<UiExpressionOutcome>, outcome: UiExpressionOutcome) {
    match settled {
        Some(held) if severity(held) >= severity(&outcome) => {}
        Some(_) | None => *settled = Some(outcome),
    }
}

/// How much of the truth an outcome withholds: a denial says no value can
/// exist, unavailability says none exists yet, staleness says the last one is
/// old.
const fn severity(outcome: &UiExpressionOutcome) -> u8 {
    match outcome {
        UiExpressionOutcome::Condition(_) | UiExpressionOutcome::Value(_) => 0,
        UiExpressionOutcome::Stale { .. } => 1,
        UiExpressionOutcome::Unavailable(_) => 2,
        UiExpressionOutcome::Denied(_) => 3,
    }
}

fn observe_query(
    name: &str,
    view: &WorthUiQueryViewIdentity,
    slot: UiProjectionInputSlot,
    inputs: &UiExpressionInputs<'_>,
) -> (UiExpressionOperandFact, Observation) {
    let Some(reference) = inputs.mounted.current_projection_input(slot) else {
        return (
            UiExpressionOperandFact::Absent {
                operand: name.into(),
            },
            Observation::NonCurrent(UiExpressionOutcome::Unavailable(
                UiExpressionUnavailableReason::ProjectionAbsent {
                    operand: name.into(),
                },
            )),
        );
    };
    let observation = query_observation(name, view, &reference);
    (UiExpressionOperandFact::Query(reference), observation)
}

fn query_observation(
    name: &str,
    view: &WorthUiQueryViewIdentity,
    reference: &UiProjectionInputFactReference,
) -> Observation {
    if reference.revision().projection_identity() != view {
        return denied(UiExpressionDenialReason::OperandProjectionMismatch {
            operand: name.into(),
        });
    }
    posture_observation(name, reference.posture(), || {
        current_scalar(name, reference)
    })
}

fn current_scalar(name: &str, reference: &UiProjectionInputFactReference) -> Observation {
    match reference {
        UiProjectionInputFactReference::Scalar(fact) => match fact.value_reference() {
            Some(text) => Observation::Bound(ExpressionValue::string(text)),
            None => denied(UiExpressionDenialReason::OperandValueMissing {
                operand: name.into(),
            }),
        },
        UiProjectionInputFactReference::Collection(_) => {
            denied(UiExpressionDenialReason::OperandShapeMismatch {
                operand: name.into(),
            })
        }
    }
}

/// Maps one projection input posture to what an expression may do with it.
/// Only a current posture yields a value, and only through `current`.
fn posture_observation(
    name: &str,
    posture: UiProjectionInputPosture,
    current: impl FnOnce() -> Observation,
) -> Observation {
    match posture {
        UiProjectionInputPosture::Current => current(),
        UiProjectionInputPosture::RetainedStale(kind) => {
            Observation::NonCurrent(UiExpressionOutcome::Stale {
                reason: UiExpressionStaleReason::ProjectionRetained {
                    operand: name.into(),
                    kind,
                },
                last_current: None,
            })
        }
        UiProjectionInputPosture::Unavailable(kind) => Observation::NonCurrent(
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Projection {
                operand: name.into(),
                kind,
            }),
        ),
        UiProjectionInputPosture::Stopped(UiProjectionFactStopKind::WrongWorld) => {
            denied(UiExpressionDenialReason::OperandWrongWorld {
                operand: name.into(),
            })
        }
        UiProjectionInputPosture::Stopped(kind) => {
            denied(UiExpressionDenialReason::OperandStopped {
                operand: name.into(),
                kind: UiExpressionStopKind::Projection(kind),
            })
        }
        UiProjectionInputPosture::TransitionStopped(kind) => {
            denied(UiExpressionDenialReason::OperandStopped {
                operand: name.into(),
                kind: UiExpressionStopKind::Transition(kind),
            })
        }
    }
}

fn observe_application(
    name: &str,
    fact: &str,
    slot: UiIntentApplicationFactSlot,
    kind: UiIntentPayloadFieldKind,
    inputs: &UiExpressionInputs<'_>,
) -> (UiExpressionOperandFact, Observation) {
    let Some(reference) = inputs.facts.input_reference(slot, inputs.generation) else {
        // The fact owner of this generation holds no value for the slot, so
        // there is nothing to read yet. That is absence, never a mismatch.
        return (
            UiExpressionOperandFact::Absent {
                operand: name.into(),
            },
            Observation::NonCurrent(UiExpressionOutcome::Unavailable(
                UiExpressionUnavailableReason::ApplicationFactAbsent {
                    operand: name.into(),
                    fact: fact.into(),
                },
            )),
        );
    };
    let observation = if reference.kind() == kind {
        match &reference {
            UiIntentApplicationInputReference::Boolean { value, .. } => {
                Observation::Bound(ExpressionValue::bool(*value))
            }
            UiIntentApplicationInputReference::Unsigned64 { value, .. } => {
                Observation::Bound(ExpressionValue::integer(i128::from(*value)))
            }
            UiIntentApplicationInputReference::Text { value, .. } => {
                Observation::Bound(ExpressionValue::string(Arc::clone(value)))
            }
        }
    } else {
        denied(UiExpressionDenialReason::OperandShapeMismatch {
            operand: name.into(),
        })
    };
    (UiExpressionOperandFact::Application(reference), observation)
}

fn observe_upstream(
    name: &str,
    identity: &str,
    upstream: UiExpressionSlot,
    records: &UiExpressionRecords,
) -> (UiExpressionOperandFact, Observation) {
    let identity: Box<str> = identity.into();
    let Some(record) = records.get(upstream) else {
        return (
            UiExpressionOperandFact::Absent {
                operand: name.into(),
            },
            Observation::NonCurrent(UiExpressionOutcome::Unavailable(
                UiExpressionUnavailableReason::Upstream {
                    operand: name.into(),
                    identity,
                },
            )),
        );
    };
    let fact = UiExpressionOperandFact::Expression {
        slot: upstream,
        outcome_revision: record.outcome_revision(),
    };
    let observation = match record.outcome() {
        UiExpressionOutcome::Condition(value) => Observation::Bound(ExpressionValue::bool(*value)),
        UiExpressionOutcome::Value(value) => Observation::Bound(value.clone()),
        UiExpressionOutcome::Denied(_) => denied(UiExpressionDenialReason::Upstream {
            operand: name.into(),
            identity,
        }),
        UiExpressionOutcome::Unavailable(_) => Observation::NonCurrent(
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Upstream {
                operand: name.into(),
                identity,
            }),
        ),
        UiExpressionOutcome::Stale { .. } => Observation::NonCurrent(UiExpressionOutcome::Stale {
            reason: UiExpressionStaleReason::Upstream {
                operand: name.into(),
                identity,
            },
            last_current: None,
        }),
    };
    (fact, observation)
}

fn denied(reason: UiExpressionDenialReason) -> Observation {
    Observation::NonCurrent(UiExpressionOutcome::Denied(reason))
}
