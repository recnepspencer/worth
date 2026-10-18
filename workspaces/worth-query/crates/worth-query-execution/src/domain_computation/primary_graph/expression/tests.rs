//! Current expression evaluation and complete operand-source identity.

use worth_foundational::expression_api::{
    expressions, ExpressionDenialFamily, ExpressionType, ExpressionValue,
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowCondition;

use super::{evaluate_condition, supporting_identity};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowConditionExpression;

fn draft(source: &str) -> CompiledWorkflowConditionExpression {
    let draft = expressions()
        .reading_within(ApplicationWorkflowCondition::PROFILE)
        .parse(source)
        .expect("the test source parses");
    CompiledWorkflowConditionExpression::Draft(draft.encode().into())
}

#[test]
fn current_expression_reads_bool_and_refuses_wrong_type() {
    for value in [true, false] {
        assert_eq!(
            evaluate_condition(
                &draft("verdict"),
                vec![(
                    "verdict",
                    &ExpressionType::Bool,
                    ExpressionValue::bool(value)
                )]
            )
            .ok(),
            Some(value)
        );
    }
    assert_eq!(
        evaluate_condition(
            &draft("verdict"),
            vec![(
                "verdict",
                &ExpressionType::INT64,
                ExpressionValue::integer(1)
            )]
        )
        .map_err(|cause| cause.family()),
        Err(ExpressionDenialFamily::TypeMismatch)
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
    let single = [("verdict", [7; 32])];
    assert_ne!(supporting_identity(&draft("verdict"), single), [7; 32]);
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
