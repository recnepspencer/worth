use std::path::Path;

use super::expression_compilation::{compile_expressions, expression};
use crate::source::{
    tokenize_module_source, WorthUiParseDiagnosticCode, WorthUiSourceModuleId, WorthUiSourceToken,
    WorthUiSourceTokenKind,
};
use crate::{WorthUiDslCompileDiagnosticCode, WorthUiExpressionIntroducer};

fn tokenize(
    source: &str,
) -> Result<Vec<WorthUiSourceToken>, Vec<crate::source::WorthUiParseDiagnostic>> {
    let module_id = WorthUiSourceModuleId::from_relative_path(Path::new("main.wui")).unwrap();
    tokenize_module_source(&module_id, source)
}

fn bodies(source: &str) -> Vec<(WorthUiExpressionIntroducer, String, usize, usize)> {
    tokenize(source)
        .expect("source should tokenize")
        .iter()
        .filter_map(|token| match token.kind() {
            WorthUiSourceTokenKind::ExpressionBody(body) => Some((
                body.introducer(),
                body.source().to_owned(),
                token.span().start_byte(),
                token.span().end_byte(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn a_body_is_one_token_covering_the_text_between_the_outer_parentheses() {
    let source = "when (a && (b || c))";
    let found = bodies(source);

    assert_eq!(found.len(), 1);
    let (introducer, text, start, end) = &found[0];
    assert_eq!(*introducer, WorthUiExpressionIntroducer::When);
    assert_eq!(text, "a && (b || c)");
    assert_eq!(&source[*start..*end], "a && (b || c)");
}

#[test]
fn value_and_whitespace_before_the_parenthesis_are_recognized() {
    let found = bodies("value\n   (x + 1)");

    assert_eq!(found[0].0, WorthUiExpressionIntroducer::Value);
    assert_eq!(found[0].1, "x + 1");
}

#[test]
fn strings_and_comments_may_hide_parentheses_and_line_comment_markers() {
    for (source, expected) in [
        (r#"when (label == ")")"#, r#"label == ")""#),
        (r#"when (label == "//")"#, r#"label == "//""#),
        (r#"when (label == "a\")b")"#, r#"label == "a\")b""#),
        (
            "when (ready // a ) here\n && ready)",
            "ready // a ) here\n && ready",
        ),
        (
            "when (xs.all(x, [x].any(y, {y}.len() > 0)))",
            "xs.all(x, [x].any(y, {y}.len() > 0))",
        ),
    ] {
        let found = bodies(source);
        assert_eq!(found.len(), 1, "{source}");
        assert_eq!(found[0].1, expected, "{source}");
    }
}

#[test]
fn an_unterminated_or_mismatched_body_reports_at_the_opening_parenthesis() {
    for source in [
        "when (a && b",
        "when (a && (b)",
        "when (a ]",
        "value (\"open)",
    ] {
        let diagnostics = tokenize(source).expect_err("body should not terminate");
        assert_eq!(
            diagnostics[0].code(),
            WorthUiParseDiagnosticCode::UnterminatedExpressionBody,
            "{source}"
        );
        assert_eq!(
            diagnostics[0].span().start_byte(),
            source.find('(').unwrap()
        );
        assert_eq!(
            diagnostics[0].span().end_byte(),
            source.find('(').unwrap() + 1
        );
    }
}

#[test]
fn appearance_when_cells_and_other_uses_of_the_words_keep_lexing_as_identifiers() {
    let source = "cell hovered when hover = hovered use token(control.background) value when";
    assert!(bodies(source).is_empty());
    let tokens = tokenize(source).expect("appearance text should tokenize");
    let identifiers = tokens
        .iter()
        .filter(|token| matches!(token.kind(), WorthUiSourceTokenKind::Identifier(text) if text == "when" || text == "value"))
        .count();
    assert_eq!(identifiers, 3);
}

#[test]
fn diagnostic_and_body_offsets_are_host_file_offsets_on_a_later_line() {
    let declaration = |body: &str| {
        format!(
            "

   condition pulse.on {{
      operand ready query-scalar pulse.ready;
      when ({body})
   }}
"
        )
    };
    let projections = super::expression_compilation::PROJECTIONS;

    let denied_source = format!("{projections}{}", declaration("ready && missing"));
    let report = super::expression_compilation::compile_source(&denied_source)
        .expect_err("an unknown binding should be denied");
    let span = report.diagnostics()[0]
        .identity()
        .span()
        .expect("a parsed declaration diagnostic has a span");
    assert_eq!(
        &denied_source[span.start_byte()..span.end_byte()],
        "missing"
    );

    let valid = declaration("ready");
    let package = compile_expressions(&valid);
    let source = format!(
        "{projections}
{valid}"
    );
    let body = expression(&package, "pulse.on")
        .body_span()
        .expect("a file-authored expression has a body span");
    assert_eq!(&source[body.start_byte()..body.end_byte()], "ready");
}

#[test]
fn a_kernel_syntax_denial_on_a_later_line_maps_to_the_host_byte_range() {
    let declaration = "

   condition pulse.on {
      operand ready query-scalar pulse.ready;
      when (ready &&
      )
   }
";
    let source = format!(
        "{}{declaration}",
        super::expression_compilation::PROJECTIONS
    );
    let report = super::expression_compilation::compile_source(&source)
        .expect_err("a dangling operator should be a syntax denial");
    let syntax = report
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.identity().code() == WorthUiDslCompileDiagnosticCode::ExpressionSyntax
        })
        .expect("a kernel syntax denial");
    let span = syntax
        .identity()
        .span()
        .expect("a parsed declaration has a span");

    // The kernel reports the missing operand where the body text ends: on
    // the last line, just before the closing parenthesis.
    let body_end = source
        .find(
            "      )
",
        )
        .expect("the closing line")
        + "      ".len();
    assert_eq!((span.start_byte(), span.end_byte()), (body_end, body_end));
    assert_eq!(&source[body_end..=body_end], ")");
}
