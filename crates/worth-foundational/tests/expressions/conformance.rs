//! V1 grammar conformance: every production parses to the documented shape,
//! and syntax outside the grammar is denied rather than implementation-defined.

use worth_foundational::expression_api::{
    expressions, ExpressionDenialDetail, ExpressionDenialFamily as Family, ExpressionOccurrence,
    ExpressionType, IntegerType, SourceOrigin, SyntaxDenial,
};

use super::{admit, admitted_type, denied_family, length, nominal};

fn syntax_denial(source: &str) -> SyntaxDenial {
    match expressions().parse(source) {
        Ok(_) => panic!("{source} parsed"),
        Err(denial) => match denial.detail() {
            ExpressionDenialDetail::Syntax(syntax) => syntax.clone(),
            other => panic!("{source}: expected a syntax denial, found {other:?}"),
        },
    }
}

#[test]
fn embedded_ui_slices_admit() {
    assert_eq!(admitted_type("ready && !ready"), ExpressionType::Bool);
    assert_eq!(
        admitted_type(r#"label ?? "Nothing selected""#),
        ExpressionType::String
    );
    assert_eq!(
        admitted_type("quantity(900.0, mm) <= clear_width"),
        ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"enable == logic4("1")"#),
        ExpressionType::Bool
    );
}

#[test]
fn precedence_follows_the_grammar() {
    // `??` is right associative and binds looser than `||`.
    assert_eq!(
        admitted_type("label ?? label ?? \"x\""),
        ExpressionType::String
    );
    // `?:` binds looser than `??`.
    assert_eq!(
        admitted_type("ready ? label ?? \"a\" : \"b\""),
        ExpressionType::String
    );
    // `*` binds tighter than `+`, and unary tighter than `*`.
    assert_eq!(
        admitted_type("-width * 2.0 + depth"),
        ExpressionType::Float64
    );
    assert_eq!(
        syntax_denial("width < depth < width"),
        SyntaxDenial::ChainedComparison
    );
    assert_eq!(
        syntax_denial("ready == ready == ready"),
        SyntaxDenial::ChainedEquality
    );
}

#[test]
fn literals_cover_the_documented_forms() {
    assert_eq!(admitted_type("0x7f + count"), ExpressionType::INT64);
    assert_eq!(admitted_type("1.5e3 + width"), ExpressionType::Float64);
    assert_eq!(
        admitted_type("int64(-9223372036854775808)"),
        ExpressionType::Integer(IntegerType::Int64)
    );
    assert_eq!(
        admitted_type(r#""tab\t\u{1F600}\"" == "x""#),
        ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"decimal("12.50")"#),
        ExpressionType::Decimal
    );
    assert_eq!(admitted_type(r#"bytes("00ff")"#), ExpressionType::Bytes);
    assert_eq!(
        admitted_type(r#"logic4("01XZ")"#),
        ExpressionType::Logic4(4)
    );
    assert_eq!(admitted_type("float32(1.5)"), ExpressionType::Float32);
    assert_eq!(
        admitted_type("width // a comment\n + 1.0"),
        ExpressionType::Float64
    );
    assert_eq!(syntax_denial(r#""\q""#), SyntaxDenial::InvalidEscape);
    assert_eq!(syntax_denial(r#""open"#), SyntaxDenial::UnterminatedString);
    assert_eq!(
        syntax_denial("width $ depth"),
        SyntaxDenial::UnexpectedCharacter('$')
    );
}

#[test]
fn collections_records_and_options_have_literal_forms() {
    assert_eq!(
        admitted_type("[1.0, 2.0, 3.0,]"),
        ExpressionType::list(ExpressionType::Float64)
    );
    assert_eq!(
        admitted_type(r#"{"a": 1.0, "b": 2.0}"#),
        ExpressionType::map(ExpressionType::String, ExpressionType::Float64)
    );
    assert_eq!(
        admitted_type("none<Float64> ?? width"),
        ExpressionType::Float64
    );
    let record = "Frame { thickness: quantity(1.0, mm), material: Material::Steel }";
    assert_eq!(
        admitted_type(record),
        ExpressionType::Record(nominal("Frame"))
    );
    assert_eq!(admitted_type(&format!("({record}).thickness")), length());
    assert_eq!(
        denied_family("Frame { thickness: quantity(1.0, mm) }"),
        Family::MissingOperand
    );
    assert_eq!(denied_family("[1.0, \"two\"]"), Family::TypeMismatch);
}

#[test]
fn record_field_order_is_schema_order() {
    let authored =
        admit("Frame { thickness: clear_width, material: Material::Steel }").expect("admits");
    let rearranged =
        admit("Frame { material: Material::Steel, thickness: clear_width }").expect("admits");
    assert_eq!(authored.identity(), rearranged.identity());
}

#[test]
fn diagnostics_carry_source_spans_not_meaning() {
    let denial = admit("width + ready").expect_err("Float64 + Bool is denied");
    match denial.occurrence() {
        Some(ExpressionOccurrence::Source(span)) => assert!(span.start() < span.end()),
        other => panic!("expected a source span, found {other:?}"),
    }
    let admitted = admit("width * 2.0").expect("admits");
    let origins: Vec<_> = (0..admitted.source_map().len())
        .filter_map(|index| admitted.source_map().origin(index))
        .collect();
    assert!(!origins.is_empty());
    assert!(origins
        .iter()
        .all(|origin: &SourceOrigin| origin.span().is_some()));
}

#[test]
fn unsupported_syntax_is_denied() {
    for source in [
        "width ** 2.0",
        "width ^ 2.0",
        "if ready then 1.0",
        "width;",
        "a.b.",
        "f(,)",
    ] {
        assert_eq!(
            expressions().parse(source).unwrap_err().family(),
            Family::Syntax,
            "{source}"
        );
    }
}
