use super::expression_compilation::{compile_expressions, expression};
use crate::{
    WorthUiAuthoredMode, WorthUiDslCompiler, WorthUiExpressionOperand,
    WorthUiExpressionOperandSource, WorthUiExpressionResultType, WorthUiProjectionLifecycle,
    WorthUiProjectionNativeFamily, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule,
};

const BASELINE: &str = "condition pulse.on {
    operand ready query-scalar pulse.ready;
    operand enabled application-boolean app.enabled;
    when (ready && enabled)
}";

#[test]
fn whitespace_comments_and_operand_order_do_not_change_identity() {
    let reformatted = "condition   pulse.on
{
    operand enabled   application-boolean app.enabled ;
    // a comment that is trivia
    operand ready query-scalar pulse.ready ;
    when (
        ready   // trailing comment
        &&
        enabled
    )
}";
    let baseline = compile_expressions(BASELINE);
    let other = compile_expressions(reformatted);

    assert_eq!(
        expression(&baseline, "pulse.on"),
        expression(&other, "pulse.on")
    );
    assert_eq!(baseline.identity(), other.identity());
}

#[test]
fn a_meaning_edit_changes_expression_and_package_identity() {
    let baseline = compile_expressions(BASELINE);
    for edited in [
        BASELINE.replace("ready && enabled", "ready || enabled"),
        BASELINE.replace(
            "application-boolean app.enabled",
            "application-boolean app.other",
        ),
        BASELINE.replace("pulse.on", "pulse.off"),
    ] {
        let package = compile_expressions(&edited);
        assert_ne!(package.identity(), baseline.identity(), "{edited}");
    }
    let disabled = compile_expressions(&BASELINE.replace("ready && enabled", "ready || enabled"));
    assert_ne!(
        expression(&disabled, "pulse.on").program_identity(),
        expression(&baseline, "pulse.on").program_identity()
    );
}

#[test]
fn file_and_rust_authored_declarations_seal_to_equal_expressions() {
    let file = compile_expressions(
        "condition pulse.on {
            operand ready query-scalar pulse.ready;
            operand enabled application-boolean app.enabled;
            when (ready && enabled)
        }
        derived pulse.count {
            operand n application-unsigned64 app.count;
            result integer;
            value (exact_cast<Int64>(n) + 1)
        }",
    );
    let module = WorthUiRustAuthoredArtifactInputModule::new("main.wui")
        .try_with_query_scalar_text(
            "pulse.label",
            "pulse.label",
            "label",
            WorthUiProjectionLifecycle::Live,
        )
        .unwrap()
        .try_with_query_scalar_native(
            "pulse.ready",
            "pulse.ready",
            "ready",
            WorthUiProjectionNativeFamily::Boolean,
            WorthUiProjectionLifecycle::Live,
        )
        .unwrap()
        .try_with_derived(
            "pulse.count",
            [WorthUiExpressionOperand::new(
                "n",
                WorthUiExpressionOperandSource::ApplicationUnsigned64 {
                    fact: "app.count".to_owned(),
                },
            )],
            WorthUiExpressionResultType::Integer,
            "exact_cast<Int64>(n) + 1",
        )
        .unwrap()
        .try_with_condition(
            "pulse.on",
            [
                WorthUiExpressionOperand::new(
                    "enabled",
                    WorthUiExpressionOperandSource::ApplicationBoolean {
                        fact: "app.enabled".to_owned(),
                    },
                ),
                WorthUiExpressionOperand::new(
                    "ready",
                    WorthUiExpressionOperandSource::QueryScalar {
                        projection: "pulse.ready".to_owned(),
                    },
                ),
            ],
            "ready && enabled",
        )
        .unwrap();
    let rust = WorthUiDslCompiler::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([module]),
    )
    .expect("Rust-authored expressions should compile through the canonical pipeline");

    assert_eq!(rust.authored_mode(), WorthUiAuthoredMode::Rust);
    for identity in ["pulse.on", "pulse.count"] {
        assert_eq!(expression(&file, identity), expression(&rust, identity));
        assert_eq!(
            expression(&file, identity).draft(),
            expression(&rust, identity).draft()
        );
    }
    assert!(expression(&rust, "pulse.on").body_span().is_none());
}

#[test]
fn rust_authored_declarations_reject_malformed_input_before_compiling() {
    let module = WorthUiRustAuthoredArtifactInputModule::new("main.wui");
    let operand = || {
        WorthUiExpressionOperand::new(
            "ready",
            WorthUiExpressionOperandSource::ApplicationBoolean {
                fact: "app.ready".to_owned(),
            },
        )
    };

    assert!(module
        .clone()
        .try_with_condition("", [operand()], "ready")
        .is_err());
    assert!(module
        .clone()
        .try_with_condition("pulse.x", [operand(), operand()], "ready")
        .is_err());
    assert!(module
        .try_with_condition(
            "pulse.x",
            [WorthUiExpressionOperand::new(
                "not a name",
                WorthUiExpressionOperandSource::ApplicationBoolean {
                    fact: "app.ready".to_owned(),
                },
            )],
            "ready",
        )
        .is_err());
}
