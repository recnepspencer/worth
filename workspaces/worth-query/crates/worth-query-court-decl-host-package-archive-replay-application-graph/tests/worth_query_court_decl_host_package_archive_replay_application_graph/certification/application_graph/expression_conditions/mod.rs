pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::expression_conditions::*;

#[test]
fn true_expression_selects_satisfied_and_its_retry_replays() {
    let court = Court::open(5, 9_176_100);
    let first = accept!(court, 9_176_110, days: days!(court.application), retained: retained!(court.application))
        .expect("a true expression settles");
    match first {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-retention");
            assert!(!performed.replayed());
        }
        other => panic!("expected a condition settlement, got {other:?}"),
    }
    let retry = accept!(court, 9_176_110, days: days!(court.application), retained: retained!(court.application))
        .expect("the retried acceptance resolves");
    match retry {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-retention");
            assert!(performed.replayed());
        }
        other => panic!("expected a replayed settlement, got {other:?}"),
    }
    court.assert_terminal(9_176_111, "satisfied");
}

#[test]
fn false_expression_selects_unsatisfied() {
    let court = Court::open(8, 9_176_200);
    let days = days!(court.application);
    let retained = retained!(court.application);
    assert_eq!(days.rows(), &[8]);
    assert_eq!(retained.rows(), &[true]);
    assert!(matches!(
        accept!(court, 9_176_210, days: days, retained: retained),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_211, "unsatisfied");
}

#[test]
fn division_by_zero_denies_without_selecting_false() {
    let court = Court::open(0, 9_176_300);
    let denied = accept!(court, 9_176_310, days: days!(court.application), retained: retained!(court.application));
    let Err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt(denial)) = denied else {
        panic!("a denied expression must refuse the acceptance: {denied:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::WorkflowConditionExpressionDenied
    );
    assert_eq!(
        denial.expression().map(|expression| expression.family()),
        Some(ExpressionDenialFamily::DivisionByZero)
    );
    court.assert_still_awaiting(9_176_311);
}

#[test]
fn absent_or_misnamed_operands_are_not_the_requirement() {
    let court = Court::open(5, 9_176_400);
    let only_retained = prepare!(court, 9_176_410, |prepared| prepared
        .expect("the condition acceptance prepares")
        .condition(&court.required)
        .operand::<DocumentRetentionConditionQueryBinding, _>(
            "retained",
            retained!(court.application),
        )
        .accept());
    assert!(matches!(
        only_retained,
        Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch)
    ));
    let misnamed = prepare!(court, 9_176_411, |prepared| prepared
        .expect("the condition acceptance prepares")
        .condition(&court.required)
        .operand::<DocumentRetentionDaysQueryBinding, _>("retention", days!(court.application))
        .operand::<DocumentRetentionConditionQueryBinding, _>(
            "retained",
            retained!(court.application),
        )
        .accept());
    assert!(matches!(
        misnamed,
        Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch)
    ));
    court.assert_still_awaiting(9_176_412);
}

#[test]
fn stale_operand_denies_and_fresh_operands_decide() {
    let court = Court::open(5, 9_176_500);
    let days = days!(court.application);
    let retained = retained!(court.application);
    set_days(&court.application, 8, 9_176_510);
    assert_eq!(
        attempt_kind(accept!(court, 9_176_511, days: days, retained: retained)),
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    );
    court.assert_still_awaiting(9_176_512);
    assert!(matches!(
        accept!(court, 9_176_513, days: days!(court.application), retained: retained!(court.application)),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_514, "unsatisfied");
}

#[test]
fn source_change_after_admission_is_fenced_at_publication() {
    let court = Court::open(5, 9_176_600);
    let days = days!(court.application);
    let retained = retained!(court.application);
    let raced = prepare!(court, 9_176_610, |prepared| {
        let admitted = prepared.expect("the condition acceptance prepares");
        set_days(&court.application, 8, 9_176_611);
        admitted
            .condition(&court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", retained)
            .accept()
    });
    match raced {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Stale(
            stale,
        ))) => assert!(stale.stale_fact_count() > 0),
        other => panic!("a transition over a replaced source must not publish: {other:?}"),
    }
    court.assert_still_awaiting(9_176_612);
}

#[test]
fn stale_zero_divisor_denies_as_stale_before_evaluation() {
    let court = Court::open(0, 9_176_900);
    let days = days!(court.application);
    let retained = retained!(court.application);
    set_days(&court.application, 5, 9_176_910);
    assert_eq!(
        attempt_kind(accept!(court, 9_176_911, days: days, retained: retained)),
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    );
    court.assert_still_awaiting(9_176_912);
}

#[test]
fn revoked_authority_cannot_accept_the_condition() {
    let court = Court::open(5, 9_176_700);
    change_advance_grant(&court, "revoked", 9_176_710);
    assert!(prepare!(court, 9_176_711, |prepared| matches!(
        prepared,
        Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(_))
    )));
    change_advance_grant(&court, "active", 9_176_712);
    assert!(matches!(
        accept!(court, 9_176_713, days: days!(court.application), retained: retained!(court.application)),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    court.assert_terminal(9_176_714, "satisfied");
}

#[test]
fn foreign_world_operands_are_refused() {
    let court = Court::open(SEED_RETENTION, 9_176_800);
    let foreign = publish_workflow_on_first_program();
    assert_eq!(
        attempt_kind(accept!(court, 9_176_810, days: days!(foreign), retained: retained!(foreign))),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    );
    court.assert_still_awaiting(9_176_811);
}

pub(crate) mod races;
