use worth_foundational::expression_api::{ExpressionType, IntegerType};

use super::expression_compilation::{compile_expressions, expression};
use crate::{WorthUiExpressionOperandSource, WorthUiExpressionResultType, WorthUiExpressionRole};

#[test]
fn a_condition_reads_every_operand_kind_and_is_boolean() {
    let package = compile_expressions(
        r#"
        condition pulse.base {
            operand ready query-scalar pulse.ready;
            when (ready)
        }
        condition pulse.healthy {
            operand base condition pulse.base;
            operand label query-scalar pulse.label;
            operand enabled application-boolean app.enabled;
            operand count application-unsigned64 app.count;
            operand name application-text app.name;
            when (base && enabled && count == count && label == name)
        }
        "#,
    );
    let healthy = expression(&package, "pulse.healthy");

    assert_eq!(healthy.role(), WorthUiExpressionRole::Condition);
    assert_eq!(healthy.admitted().result_type(), &ExpressionType::Bool);
    let operands = healthy
        .operands()
        .iter()
        .map(|operand| (operand.name(), operand.expression_type().clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        operands,
        [
            ("base", ExpressionType::Bool),
            ("count", ExpressionType::Integer(IntegerType::UInt64)),
            ("enabled", ExpressionType::Bool),
            ("label", ExpressionType::String),
            ("name", ExpressionType::String),
        ]
    );
    assert_eq!(package.expressions().len(), 2);
}

#[test]
fn a_derived_yields_each_declared_result_type() {
    let package = compile_expressions(
        r#"
        derived pulse.text {
            operand label query-scalar pulse.label;
            result text;
            value (label)
        }
        derived pulse.integer {
            operand count application-unsigned64 app.count;
            result integer;
            value (exact_cast<Int64>(count) + 1)
        }
        derived pulse.decimal {
            operand name application-text app.name;
            result decimal;
            value (name == "x" ? decimal("1.5") : decimal("2.5"))
        }
        derived pulse.token {
            operand label query-scalar pulse.label;
            result token;
            value (label == "on" ? "active" : label)
        }
        "#,
    );
    let results = [
        (
            "pulse.text",
            WorthUiExpressionResultType::Text,
            ExpressionType::String,
        ),
        (
            "pulse.integer",
            WorthUiExpressionResultType::Integer,
            ExpressionType::INT64,
        ),
        (
            "pulse.decimal",
            WorthUiExpressionResultType::Decimal,
            ExpressionType::Decimal,
        ),
        (
            "pulse.token",
            WorthUiExpressionResultType::Token,
            ExpressionType::String,
        ),
    ];
    for (identity, result, expected) in results {
        let sealed = expression(&package, identity);
        assert_eq!(sealed.role(), WorthUiExpressionRole::Derived(result));
        assert_eq!(sealed.admitted().result_type(), &expected);
    }
}

#[test]
fn an_expression_can_consume_a_derived_and_a_condition() {
    let package = compile_expressions(
        r#"
        derived pulse.doubled {
            operand count application-unsigned64 app.count;
            result integer;
            value (exact_cast<Int64>(count) * 2)
        }
        condition pulse.big {
            operand doubled derived pulse.doubled;
            when (doubled > 10)
        }
        derived pulse.label-or-flag {
            operand big condition pulse.big;
            operand label query-scalar pulse.label;
            result text;
            value (big ? label : "quiet")
        }
        "#,
    );
    let big = expression(&package, "pulse.big");

    assert_eq!(
        big.operands()[0].source(),
        &WorthUiExpressionOperandSource::Derived {
            identity: "pulse.doubled".to_owned()
        }
    );
    assert_eq!(big.operands()[0].expression_type(), &ExpressionType::INT64);
    assert_eq!(package.expressions().len(), 3);
}

#[test]
fn sealed_expressions_carry_the_draft_and_program_identity() {
    let package = compile_expressions(
        "condition pulse.on { operand ready query-scalar pulse.ready; when (ready) }",
    );
    let on = expression(&package, "pulse.on");

    assert!(!on.draft().is_empty());
    assert_eq!(on.admitted().identity(), on.program_identity());
    assert_eq!(on.schema().operands().len(), 1);
    assert!(on.body_span().is_some());
}
