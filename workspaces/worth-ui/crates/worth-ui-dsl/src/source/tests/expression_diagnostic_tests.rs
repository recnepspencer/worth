use super::expression_compilation::{compile_source, denials, PROJECTIONS};
use crate::WorthUiDslCompileDiagnosticCode as Code;
use crate::WorthUiDslCompileStopClass;

/// Asserts one diagnostic carries `expected` and a message naming the rule
/// that fired, so a rule cannot be replaced by another that shares its code.
fn assert_denied(declarations: &str, expected: Code, message_fragment: &str) {
    let found = denials(declarations);
    assert!(
        found
            .iter()
            .any(|(code, message)| *code == expected && message.contains(message_fragment)),
        "expected {expected:?} mentioning `{message_fragment}` in {found:?} for {declarations}"
    );
}

#[test]
fn expression_to_expression_cycles_are_named_including_self_reference() {
    let report = compile_source(&format!(
        "{PROJECTIONS}
        condition pulse.a {{ operand b condition pulse.b; when (b) }}
        condition pulse.b {{ operand a condition pulse.a; when (a) }}
        condition pulse.self {{ operand me condition pulse.self; when (me) }}"
    ))
    .expect_err("cycles should be denied");
    let messages = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.identity().code() == Code::ExpressionCycle)
        .map(|diagnostic| diagnostic.message().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(messages.len(), 2, "{messages:?}");
    assert!(messages
        .iter()
        .any(|message| message.contains("pulse.a -> pulse.b -> pulse.a")));
    assert!(messages
        .iter()
        .any(|message| message.contains("pulse.self -> pulse.self")));
}

#[test]
fn operand_sources_that_do_not_resolve_are_diagnosed_by_name() {
    assert_denied(
        "condition pulse.x { operand v query-scalar pulse.missing; when (v) }",
        Code::UnknownExpressionOperandSource,
        "unknown scalar projection `pulse.missing`",
    );
    assert_denied(
        "condition pulse.x { operand v condition pulse.missing; when (v) }",
        Code::UnknownExpressionOperandSource,
        "unknown condition `pulse.missing`",
    );
    assert_denied(
        "condition pulse.x { operand v derived pulse.missing; when (v) }",
        Code::UnknownExpressionOperandSource,
        "unknown derived declaration `pulse.missing`",
    );
    assert_denied(
        "condition pulse.x { operand v application-float app.value; when (v) }",
        Code::UnknownExpressionOperandSource,
        "unknown source kind `application-float`",
    );
}

#[test]
fn invalid_declarations_name_the_rule_they_break() {
    for (declarations, fragment) in [
        (
            "condition pulse.x { operand rows query-scalar pulse.rows; when (rows == rows) }",
            "which is a collection",
        ),
        (
            "derived pulse.d { operand l query-scalar pulse.label; result text; value (l) }
             condition pulse.x { operand d condition pulse.d; when (d) }",
            "which is a derived declaration, not a condition",
        ),
        (
            "condition pulse.c { operand r query-scalar pulse.ready; when (r) }
             condition pulse.x { operand c derived pulse.c; when (c) }",
            "which is a condition, not a derived declaration",
        ),
        (
            "derived pulse.x { operand r query-scalar pulse.ready; result boolean; value (r) }",
            "cannot yield boolean; declare a `condition`",
        ),
        (
            "derived pulse.x { operand r query-scalar pulse.ready; result flag; value (r) }",
            "unknown result type `flag`",
        ),
        (
            "condition pulse.x { operand r query-scalar pulse.ready; result text; when (r) }",
            "a condition is always boolean and declares no `result`",
        ),
        (
            "derived pulse.x { operand r query-scalar pulse.ready; value (r) }",
            "a derived declaration requires `result <type>;`",
        ),
        (
            "derived pulse.x { operand r query-scalar pulse.ready; result text; result text; value (r) }",
            "`result` is declared more than once",
        ),
        (
            "condition pulse.x { operand r query-scalar pulse.ready; operand r query-scalar pulse.ready; when (r) }",
            "operand `r` is declared more than once",
        ),
        (
            "condition pulse.x { operand true query-scalar pulse.ready; when (true) }",
            "is not a valid expression operand name",
        ),
        (
            "condition pulse.x { operand r query-scalar pulse.ready; when (r) }
             condition pulse.x { operand r query-scalar pulse.ready; when (r) }",
            "`pulse.x` appears more than once",
        ),
        (
            "condition pulse.x { when (true) value (1) }",
            "must be the last clause of `pulse.x`",
        ),
        (
            "condition pulse.x { operand r query-scalar pulse.ready; }",
            "requires a `when(...)` body",
        ),
        (
            "derived pulse.x { operand r query-scalar pulse.ready; result text; when (r) }",
            "expected `value(...)` for this declaration, found `when(...)`",
        ),
        (
            "condition pulse.x { operand r query-scalar pulse.ready when (r) }",
            "`operand` clause must end with a semicolon",
        ),
    ] {
        assert_denied(declarations, Code::InvalidExpressionDeclaration, fragment);
    }
}

#[test]
fn ill_typed_and_mismatched_bodies_are_kernel_denials_naming_the_family() {
    assert_denied(
        "condition pulse.x { operand l query-scalar pulse.label; when (l + 1) }",
        Code::ExpressionAdmissionDenied,
        "denied (TypeMismatch)",
    );
    assert_denied(
        "condition pulse.x { operand l query-scalar pulse.label; when (l) }",
        Code::ExpressionAdmissionDenied,
        "denied (TypeMismatch)",
    );
    assert_denied(
        "derived pulse.x { operand l query-scalar pulse.label; result integer; value (l) }",
        Code::ExpressionAdmissionDenied,
        "denied (TypeMismatch)",
    );
    assert_denied(
        "condition pulse.x { operand r query-scalar pulse.ready; when (r && missing) }",
        Code::ExpressionAdmissionDenied,
        "denied (UnknownBinding)",
    );
    assert_denied(
        "condition pulse.x { operand r query-scalar pulse.ready; when (r && ) }",
        Code::ExpressionSyntax,
        "denied (Syntax)",
    );

    let report = compile_source(&format!(
        "{PROJECTIONS} condition pulse.x {{ operand r query-scalar pulse.ready; when (r && ) }}"
    ))
    .unwrap_err();
    let syntax = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.identity().code() == Code::ExpressionSyntax)
        .unwrap();
    assert_eq!(
        syntax.stop_class(),
        WorthUiDslCompileStopClass::LanguageSyntax
    );
}

#[test]
fn a_declared_operand_the_program_never_reads_is_diagnosed() {
    assert_denied(
        "condition pulse.x {
            operand r query-scalar pulse.ready;
            operand spare application-boolean app.spare;
            when (r)
        }",
        Code::UnusedExpressionOperand,
        "operand `spare` of expression `pulse.x` is declared but never read",
    );
}

#[test]
fn let_bindings_are_denied_in_favor_of_a_named_derived_declaration() {
    let report = compile_source(&format!(
        "{PROJECTIONS} condition pulse.x {{
            operand r query-scalar pulse.ready;
            when (let same = r; same && r)
        }}"
    ))
    .expect_err("let should be denied");
    let denial = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.identity().code() == Code::ExpressionAdmissionDenied)
        .expect("a let denial");

    assert!(
        denial.message().contains("uses a `let` binding")
            && denial.message().contains("named `derived` declaration"),
        "{}",
        denial.message()
    );
}

#[test]
fn an_unterminated_body_is_a_syntax_stop_with_its_own_code() {
    let report = compile_source("condition pulse.x { when (ready ").unwrap_err();

    assert_eq!(
        report.diagnostics()[0].identity().code(),
        Code::UnterminatedExpressionBody
    );
    assert_eq!(
        report.diagnostics()[0].stop_class(),
        WorthUiDslCompileStopClass::LanguageSyntax
    );
}

#[test]
fn one_invalid_expression_fails_the_whole_package() {
    let result = compile_source(&format!(
        "{PROJECTIONS}
        condition pulse.good {{ operand r query-scalar pulse.ready; when (r) }}
        condition pulse.bad {{ operand r query-scalar pulse.ready; when (r + 1) }}"
    ));

    assert!(result.is_err(), "no partial package is produced");
}
