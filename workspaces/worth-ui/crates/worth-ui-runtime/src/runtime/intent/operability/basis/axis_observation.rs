use crate::declaration::{
    UiIntentOperabilityDependencyAxis, UiResolvedIntentConditionSource,
    UiResolvedIntentConfirmationSource, UiResolvedIntentMutabilitySource,
    UiResolvedIntentPolicySource, UiResolvedIntentReadinessSource,
};
use crate::runtime::expression::{UiExpressionConditionWithholding, UiExpressionResultReference};
use crate::runtime::intent::payload::{UiIntentApplicationInputReference, UiIntentInputBasisView};

use super::super::{
    UiIntentConfirmationPosture, UiIntentMutabilityPosture, UiIntentPolicyPosture,
    UiIntentReadinessPosture, UiIntentWithheldCondition,
};

/// Every fact the declared axes read, retained so admission can prove the
/// decision still current.
#[derive(Default)]
pub(super) struct UiIntentAxisInputs {
    pub(super) query: Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    pub(super) application: Vec<UiIntentApplicationInputReference>,
    pub(super) expressions: Vec<(
        UiIntentOperabilityDependencyAxis,
        UiExpressionResultReference,
    )>,
}

pub(super) struct UiIntentObservedAxis<P> {
    pub(super) posture: P,
    pub(super) input: Option<UiIntentApplicationInputReference>,
}

pub(super) fn observe_mutability(
    view: &UiIntentInputBasisView<'_>,
    source: &UiResolvedIntentMutabilitySource,
    inputs: &mut UiIntentAxisInputs,
) -> UiIntentMutabilityPosture {
    match source {
        UiResolvedIntentMutabilitySource::ApplicationBoolean(slot) => {
            let (input, writable) = application_boolean(view, *slot);
            inputs.application.push(input);
            if writable {
                UiIntentMutabilityPosture::Writable
            } else {
                UiIntentMutabilityPosture::Readonly
            }
        }
        UiResolvedIntentMutabilitySource::ProjectionReadonly { identity, slot } => {
            retain_projection(view, identity, *slot, inputs);
            UiIntentMutabilityPosture::Readonly
        }
        UiResolvedIntentMutabilitySource::CommittedDraft => UiIntentMutabilityPosture::Writable,
        UiResolvedIntentMutabilitySource::Condition(condition) => {
            match observe_condition(
                view,
                condition,
                UiIntentOperabilityDependencyAxis::Mutability,
                inputs,
            ) {
                Ok(true) => UiIntentMutabilityPosture::Writable,
                Ok(false) => UiIntentMutabilityPosture::Readonly,
                Err(withheld) => UiIntentMutabilityPosture::Withheld(withheld),
            }
        }
    }
}

pub(super) fn observe_readiness(
    view: &UiIntentInputBasisView<'_>,
    source: &UiResolvedIntentReadinessSource,
    inputs: &mut UiIntentAxisInputs,
) -> UiIntentReadinessPosture {
    match source {
        UiResolvedIntentReadinessSource::ApplicationBoolean(slot) => {
            let (input, ready) = application_boolean(view, *slot);
            inputs.application.push(input);
            ready_posture(ready)
        }
        UiResolvedIntentReadinessSource::Projection { identity, slot } => {
            ready_posture(retain_projection(view, identity, *slot, inputs))
        }
        UiResolvedIntentReadinessSource::CommittedDraft => UiIntentReadinessPosture::Ready,
        UiResolvedIntentReadinessSource::Condition(condition) => {
            match observe_condition(
                view,
                condition,
                UiIntentOperabilityDependencyAxis::Readiness,
                inputs,
            ) {
                Ok(ready) => ready_posture(ready),
                Err(withheld) => UiIntentReadinessPosture::Withheld(withheld),
            }
        }
    }
}

/// A policy read from an application fact keeps that fact as its own input,
/// so admission reports its drift as a policy change.
pub(super) fn observe_policy(
    view: &UiIntentInputBasisView<'_>,
    source: &UiResolvedIntentPolicySource,
    inputs: &mut UiIntentAxisInputs,
) -> UiIntentObservedAxis<UiIntentPolicyPosture> {
    match source {
        UiResolvedIntentPolicySource::ApplicationBoolean(slot) => {
            let (input, admitted) = application_boolean(view, *slot);
            UiIntentObservedAxis {
                posture: policy_posture(admitted),
                input: Some(input),
            }
        }
        UiResolvedIntentPolicySource::Condition(condition) => UiIntentObservedAxis {
            posture: match observe_condition(
                view,
                condition,
                UiIntentOperabilityDependencyAxis::Policy,
                inputs,
            ) {
                Ok(admitted) => policy_posture(admitted),
                Err(withheld) => UiIntentPolicyPosture::Withheld(withheld),
            },
            input: None,
        },
    }
}

pub(super) fn observe_confirmation(
    view: &UiIntentInputBasisView<'_>,
    contract: &crate::declaration::UiResolvedIntentConfirmationContract,
) -> UiIntentObservedAxis<UiIntentConfirmationPosture> {
    match contract.source() {
        UiResolvedIntentConfirmationSource::NotRequired => UiIntentObservedAxis {
            posture: UiIntentConfirmationPosture::NotRequired,
            input: None,
        },
        UiResolvedIntentConfirmationSource::ApplicationBoolean(slot) => {
            let (input, required) = application_boolean(view, *slot);
            let posture = if required {
                UiIntentConfirmationPosture::Required {
                    policy_identity: contract.policy_identity().into(),
                }
            } else {
                UiIntentConfirmationPosture::NotRequired
            };
            UiIntentObservedAxis {
                posture,
                input: Some(input),
            }
        }
    }
}

/// The truth value of a condition, or the withholding that stands in for it.
/// The result read is retained whatever its posture, so a withheld axis is
/// proven stale exactly like a decided one. An owner that does not follow the
/// view's generation holds no result for it, which is stale by definition.
fn observe_condition(
    view: &UiIntentInputBasisView<'_>,
    condition: &UiResolvedIntentConditionSource,
    axis: UiIntentOperabilityDependencyAxis,
    inputs: &mut UiIntentAxisInputs,
) -> Result<bool, UiIntentWithheldCondition> {
    let slot = condition.slot();
    let truth = match view.expression(slot) {
        Some(result) => {
            let truth = result.condition();
            inputs.expressions.push((axis, result));
            truth
        }
        None => Err(UiExpressionConditionWithholding::Stale),
    };
    truth.map_err(|withholding| UiIntentWithheldCondition::new(slot, withholding))
}

/// The one read of a resolved Boolean application fact. Catalog preparation
/// proves every resolved slot exists with Boolean shape in the fact plan the
/// fact state is built from, for the generation the view observes; the fact
/// state has no typed per-kind slot yet that would carry that proof here.
fn application_boolean(
    view: &UiIntentInputBasisView<'_>,
    slot: crate::declaration::UiIntentApplicationFactSlot,
) -> (UiIntentApplicationInputReference, bool) {
    let input = view
        .application(slot)
        .expect("resolved application fact slot exists in the active fact state");
    let value = input
        .boolean_value()
        .expect("resolved operability fact has Boolean shape");
    (input, value)
}

fn retain_projection(
    view: &UiIntentInputBasisView<'_>,
    expected: &worth_ui_query_binding::WorthUiQueryViewIdentity,
    slot: worth_ui_query_binding::UiProjectionInputSlot,
    inputs: &mut UiIntentAxisInputs,
) -> bool {
    let Some(input) = view.projection(slot) else {
        return false;
    };
    let current = input.revision().projection_identity() == expected
        && input.revision().slot() == slot
        && input.posture() == worth_ui_query_binding::UiProjectionInputPosture::Current;
    inputs.query.push(input);
    current
}

const fn ready_posture(ready: bool) -> UiIntentReadinessPosture {
    if ready {
        UiIntentReadinessPosture::Ready
    } else {
        UiIntentReadinessPosture::Pending
    }
}

const fn policy_posture(admitted: bool) -> UiIntentPolicyPosture {
    if admitted {
        UiIntentPolicyPosture::Admitted
    } else {
        UiIntentPolicyPosture::Denied
    }
}
