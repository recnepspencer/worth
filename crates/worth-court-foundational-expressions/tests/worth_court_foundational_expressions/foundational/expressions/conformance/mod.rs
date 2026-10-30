// V1 grammar conformance: every production parses to the documented shape,
// and syntax outside the grammar is denied rather than implementation-defined.
#[allow(unused_imports)]
use crate::expressions::ordinary_context::conformance::*;
pub(crate) use {
    worth_foundational::expression_api::expressions,
    worth_foundational::expression_api::ExpressionDenialDetail,
    worth_foundational::expression_api::ExpressionDenialFamily as Family,
    worth_foundational::expression_api::ExpressionOccurrence,
    worth_foundational::expression_api::ExpressionType,
    worth_foundational::expression_api::IntegerType,
    worth_foundational::expression_api::SourceOrigin,
    worth_foundational::expression_api::SyntaxDenial,
};

pub(crate) use {
    crate::expressions::ordinary_context::admit,
    crate::expressions::ordinary_context::admitted_type,
    crate::expressions::ordinary_context::denied_family,
    crate::expressions::ordinary_context::length, crate::expressions::ordinary_context::nominal,
};

#[test]
fn embedded_ui_slices_admit() {
    assert_eq!(
        admitted_type("ready && !ready"),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"label ?? "Nothing selected""#),
        worth_foundational::expression_api::ExpressionType::String
    );
    assert_eq!(
        admitted_type("quantity(900.0, mm) <= clear_width"),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"case_equal(enable, logic4("1"))"#),
        worth_foundational::expression_api::ExpressionType::Bool
    );
}

#[test]
fn precedence_follows_the_grammar() {
    // `??` is right associative and binds looser than `||`.
    assert_eq!(
        admitted_type("label ?? label ?? \"x\""),
        worth_foundational::expression_api::ExpressionType::String
    );
    // `?:` binds looser than `??`.
    assert_eq!(
        admitted_type("ready ? label ?? \"a\" : \"b\""),
        worth_foundational::expression_api::ExpressionType::String
    );
    // `*` binds tighter than `+`, and unary tighter than `*`.
    assert_eq!(
        admitted_type("-width * 2.0 + depth"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    assert_eq!(
        syntax_denial("width < depth < width"),
        worth_foundational::expression_api::SyntaxDenial::ChainedComparison
    );
    assert_eq!(
        syntax_denial("ready == ready == ready"),
        worth_foundational::expression_api::SyntaxDenial::ChainedEquality
    );
}

#[test]
fn literals_cover_the_documented_forms() {
    assert_eq!(
        admitted_type("0x7f + count"),
        worth_foundational::expression_api::ExpressionType::INT64
    );
    assert_eq!(
        admitted_type("1.5e3 + width"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    assert_eq!(
        admitted_type("int64(-9223372036854775808)"),
        worth_foundational::expression_api::ExpressionType::Integer(
            worth_foundational::expression_api::IntegerType::Int64
        )
    );
    assert_eq!(
        admitted_type(r#""tab\t\u{1F600}\"" == "x""#),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"decimal("12.50")"#),
        worth_foundational::expression_api::ExpressionType::Decimal
    );
    assert_eq!(
        admitted_type(r#"bytes("00ff")"#),
        worth_foundational::expression_api::ExpressionType::Bytes
    );
    assert_eq!(
        admitted_type(r#"logic4("01XZ")"#),
        worth_foundational::expression_api::ExpressionType::Logic4(4)
    );
    assert_eq!(
        admitted_type("float32(1.5)"),
        worth_foundational::expression_api::ExpressionType::Float32
    );
    assert_eq!(
        admitted_type("width // a comment\n + 1.0"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    assert_eq!(
        syntax_denial(r#""\q""#),
        worth_foundational::expression_api::SyntaxDenial::InvalidEscape
    );
    assert_eq!(
        syntax_denial(r#""open"#),
        worth_foundational::expression_api::SyntaxDenial::UnterminatedString
    );
    assert_eq!(
        syntax_denial("width $ depth"),
        worth_foundational::expression_api::SyntaxDenial::UnexpectedCharacter('$')
    );
}

#[test]
fn collections_records_and_options_have_literal_forms() {
    assert_eq!(
        admitted_type("[1.0, 2.0, 3.0,]"),
        worth_foundational::expression_api::ExpressionType::list(
            worth_foundational::expression_api::ExpressionType::Float64
        )
    );
    assert_eq!(
        admitted_type(r#"{"a": 1.0, "b": 2.0}"#),
        worth_foundational::expression_api::ExpressionType::map(
            worth_foundational::expression_api::ExpressionType::String,
            worth_foundational::expression_api::ExpressionType::Float64
        )
    );
    assert_eq!(
        admitted_type("none<Float64> ?? width"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    let record = "Frame { thickness: quantity(1.0, mm), material: Material::Steel }";
    assert_eq!(
        admitted_type(record),
        worth_foundational::expression_api::ExpressionType::Record(nominal("Frame"))
    );
    assert_eq!(admitted_type(&format!("({record}).thickness")), length());
    assert_eq!(
        denied_family("Frame { thickness: quantity(1.0, mm) }"),
        worth_foundational::expression_api::ExpressionDenialFamily::MissingOperand
    );
    assert_eq!(
        denied_family("[1.0, \"two\"]"),
        worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
    );
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
        Some(worth_foundational::expression_api::ExpressionOccurrence::Source(span)) => {
            assert!(span.start() < span.end())
        }
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
            worth_foundational::expression_api::ExpressionDenialFamily::Syntax,
            "{source}"
        );
    }
}
