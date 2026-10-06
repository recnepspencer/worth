use super::*;

#[test]
fn only_a_truth_value_is_a_condition_and_every_other_outcome_keeps_its_withholding() {
    let operand: Box<str> = "f".into();
    for (outcome, expected) in [
        (UiExpressionOutcome::Condition(true), Ok(true)),
        (UiExpressionOutcome::Condition(false), Ok(false)),
        (
            UiExpressionOutcome::Value(ExpressionValue::bool(false)),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandShapeMismatch {
                operand: operand.clone(),
            }),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
                operand: operand.clone(),
            }),
            Err(UiExpressionWithholding::Unavailable),
        ),
        (
            UiExpressionOutcome::Stale {
                reason: UiExpressionStaleReason::ProjectionRetained {
                    operand: operand.clone(),
                    kind: UiProjectionRetainedActivityKind::Idle,
                },
                last_current: Some(UiExpressionCurrentValue::Condition(true)),
            },
            Err(UiExpressionWithholding::Stale),
        ),
    ] {
        assert_eq!(outcome.condition(), expected, "{outcome:?}");
    }
}

#[test]
fn a_condition_that_yields_a_non_boolean_is_a_role_mismatch_never_false() {
    let role = WorthUiExpressionRole::Condition;

    assert_eq!(
        UiExpressionOutcome::of_role(role, &ExpressionValue::integer(1)),
        UiExpressionOutcome::Denied(UiExpressionDenialReason::RoleMismatch { role })
    );
    assert_eq!(
        UiExpressionOutcome::of_role(role, &ExpressionValue::bool(false)),
        UiExpressionOutcome::Condition(false)
    );
}

#[test]
fn a_derived_value_of_its_declared_type_is_that_value() {
    for (result, value) in [
        (
            WorthUiExpressionResultType::Text,
            ExpressionValue::string("a"),
        ),
        (
            WorthUiExpressionResultType::Token,
            ExpressionValue::string("on"),
        ),
        (
            WorthUiExpressionResultType::Integer,
            ExpressionValue::integer(-3),
        ),
        (
            WorthUiExpressionResultType::Decimal,
            ExpressionValue::decimal_parts(15, 1).unwrap(),
        ),
    ] {
        let role = WorthUiExpressionRole::Derived(result);

        assert_eq!(
            UiExpressionOutcome::of_role(role, &value),
            UiExpressionOutcome::Value(value.clone()),
            "{result:?}"
        );
    }
}

#[test]
fn a_derived_value_of_another_kernel_type_is_a_role_mismatch_never_coerced() {
    for (result, value) in [
        (
            WorthUiExpressionResultType::Text,
            ExpressionValue::integer(1),
        ),
        (
            WorthUiExpressionResultType::Token,
            ExpressionValue::bool(true),
        ),
        (
            WorthUiExpressionResultType::Integer,
            ExpressionValue::string("1"),
        ),
        (
            WorthUiExpressionResultType::Integer,
            ExpressionValue::integer(i128::from(i64::MAX) + 1),
        ),
        (
            WorthUiExpressionResultType::Decimal,
            ExpressionValue::integer(2),
        ),
    ] {
        let role = WorthUiExpressionRole::Derived(result);

        assert_eq!(
            UiExpressionOutcome::of_role(role, &value),
            UiExpressionOutcome::Denied(UiExpressionDenialReason::RoleMismatch { role }),
            "{result:?} {value:?}"
        );
    }
}

#[test]
fn only_a_text_value_is_derived_text_and_every_other_outcome_keeps_its_withholding() {
    let operand: Box<str> = "f".into();
    for (outcome, expected) in [
        (
            UiExpressionOutcome::Value(ExpressionValue::string("ready")),
            Ok("ready"),
        ),
        (
            UiExpressionOutcome::Value(ExpressionValue::integer(1)),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Condition(true),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
                operand: operand.clone(),
            }),
            Err(UiExpressionWithholding::Unavailable),
        ),
        (
            UiExpressionOutcome::Stale {
                reason: UiExpressionStaleReason::ProjectionRetained {
                    operand: operand.clone(),
                    kind: UiProjectionRetainedActivityKind::Idle,
                },
                last_current: Some(UiExpressionCurrentValue::Value(ExpressionValue::string(
                    "old",
                ))),
            },
            Err(UiExpressionWithholding::Stale),
        ),
    ] {
        assert_eq!(outcome.derived_text(), expected, "{outcome:?}");
    }
}

#[test]
fn only_an_integer_value_is_a_derived_integer_and_every_other_outcome_keeps_its_withholding() {
    let operand: Box<str> = "f".into();
    for (outcome, expected) in [
        (
            UiExpressionOutcome::Value(ExpressionValue::integer(7)),
            Ok(7),
        ),
        (
            UiExpressionOutcome::Value(ExpressionValue::integer(-2)),
            Ok(-2),
        ),
        (
            UiExpressionOutcome::Value(ExpressionValue::string("7")),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Condition(true),
            Err(UiExpressionWithholding::Denied),
        ),
        (
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
                operand: operand.clone(),
            }),
            Err(UiExpressionWithholding::Unavailable),
        ),
        (
            UiExpressionOutcome::Stale {
                reason: UiExpressionStaleReason::ProjectionRetained {
                    operand: operand.clone(),
                    kind: UiProjectionRetainedActivityKind::Idle,
                },
                last_current: Some(UiExpressionCurrentValue::Value(ExpressionValue::integer(1))),
            },
            Err(UiExpressionWithholding::Stale),
        ),
    ] {
        assert_eq!(outcome.derived_integer(), expected, "{outcome:?}");
    }
}
