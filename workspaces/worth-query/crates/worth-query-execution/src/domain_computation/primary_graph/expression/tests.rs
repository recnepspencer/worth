//! Version-1 conditions carried into the expression evaluator: a migrated
//! condition reads its single Bool source unchanged and replays under that
//! source's own identity.

use worth_foundational::expression_api::{
    expressions, ExpressionDenialFamily, ExpressionType, ExpressionValue,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowCondition, MIGRATED_WORKFLOW_CONDITION_OPERAND,
};

use super::{evaluate_condition, supporting_identity};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowConditionExpression;

const MIGRATED: CompiledWorkflowConditionExpression = CompiledWorkflowConditionExpression::Migrated;

fn draft(source: &str) -> CompiledWorkflowConditionExpression {
    let draft = expressions()
        .reading_within(ApplicationWorkflowCondition::PROFILE)
        .parse(source)
        .expect("the test source parses");
    CompiledWorkflowConditionExpression::Draft(draft.encode().into())
}

fn migrated(value: ExpressionValue, ty: &ExpressionType) -> Result<bool, ExpressionDenialFamily> {
    evaluate_condition(
        &MIGRATED,
        vec![(MIGRATED_WORKFLOW_CONDITION_OPERAND, ty, value)],
    )
    .map_err(|denial| denial.family())
}

#[test]
fn migrated_condition_reads_its_source_verdict() {
    assert_eq!(
        migrated(ExpressionValue::bool(true), &ExpressionType::Bool),
        Ok(true)
    );
    assert_eq!(
        migrated(ExpressionValue::bool(false), &ExpressionType::Bool),
        Ok(false)
    );
}

#[test]
fn migrated_condition_matches_the_same_source_authored_as_an_expression() {
    let authored = draft(MIGRATED_WORKFLOW_CONDITION_OPERAND);
    for verdict in [true, false] {
        let operand = || {
            vec![(
                MIGRATED_WORKFLOW_CONDITION_OPERAND,
                &ExpressionType::Bool,
                ExpressionValue::bool(verdict),
            )]
        };
        assert_eq!(
            evaluate_condition(&MIGRATED, operand()).ok(),
            evaluate_condition(&authored, operand()).ok()
        );
    }
}

#[test]
fn migrated_condition_over_a_non_bool_source_denies() {
    assert_eq!(
        migrated(ExpressionValue::integer(1), &ExpressionType::INT64),
        Err(ExpressionDenialFamily::TypeMismatch)
    );
}

#[test]
fn migrated_replay_identity_is_the_single_source_identity() {
    let source = [7; 32];
    assert_eq!(
        supporting_identity(&MIGRATED, [(MIGRATED_WORKFLOW_CONDITION_OPERAND, source)]),
        source
    );
}

#[test]
fn expression_identity_binds_every_named_source() {
    let authored = draft("days > 0 && retained");
    let baseline = supporting_identity(&authored, [("days", [1; 32]), ("retained", [2; 32])]);
    assert_ne!(baseline, [1; 32]);
    assert_ne!(
        baseline,
        supporting_identity(&authored, [("days", [3; 32]), ("retained", [2; 32])])
    );
    assert_ne!(
        baseline,
        supporting_identity(&authored, [("day", [1; 32]), ("retained", [2; 32])])
    );
    let single = [(MIGRATED_WORKFLOW_CONDITION_OPERAND, [7; 32])];
    assert_ne!(
        supporting_identity(&draft(MIGRATED_WORKFLOW_CONDITION_OPERAND), single),
        supporting_identity(&MIGRATED, single)
    );
}

#[test]
fn an_absent_optional_operand_denies_rather_than_reading_false() {
    let approved = ExpressionType::option(ExpressionType::Bool);
    let absent = || vec![("approved", &approved, ExpressionValue::none())];
    assert_eq!(
        evaluate_condition(&draft("unwrap(approved)"), absent()).map_err(|denial| denial.family()),
        Err(ExpressionDenialFamily::AbsentValue)
    );
    assert_eq!(
        evaluate_condition(&draft("is_some(approved) && unwrap(approved)"), absent()).ok(),
        Some(false)
    );
}
