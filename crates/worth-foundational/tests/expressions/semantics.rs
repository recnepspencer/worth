//! Admission semantics: result types, closed operator contracts, and the
//! typed denials for width, unit, and type errors.

use worth_foundational::expression_api::{
    expressions, ExpressionDenialDetail, ExpressionDenialFamily as Family,
    ExpressionFunctionCatalog, ExpressionProfile, ExpressionType, IntegerType,
};

use super::{admitted_type, denied_family, function, length, nominal, schema};

#[test]
fn engineering_arithmetic_keeps_types_and_dimensions() {
    assert_eq!(
        admitted_type("width - 2.0 * depth"),
        ExpressionType::Float64
    );
    assert_eq!(admitted_type("count + 1"), ExpressionType::INT64);
    assert_eq!(
        admitted_type("quantity(900.0, mm) <= clear_width"),
        ExpressionType::Bool
    );
    assert_eq!(
        admitted_type("clear_width - frame.thickness * 2.0"),
        length()
    );
    assert_eq!(
        admitted_type("magnitude(clear_width, mm)"),
        ExpressionType::Float64
    );
}

#[test]
fn quantities_deny_dimension_and_scalar_mismatches() {
    assert_eq!(
        denied_family("clear_width + quantity(1.0, s)"),
        Family::TypeMismatch
    );
    assert_eq!(denied_family("clear_width + width"), Family::TypeMismatch);
    assert_eq!(
        denied_family("quantity(1.0, parsec)"),
        Family::UnknownBinding
    );
}

#[test]
fn logic_options_and_collections_type_exactly() {
    assert_eq!(admitted_type("ready && !ready"), ExpressionType::Bool);
    assert_eq!(
        admitted_type(r#"label ?? "Nothing selected""#),
        ExpressionType::String
    );
    assert_eq!(
        admitted_type("members.filter(m, m.material == Material::Steel).map(m, m.thickness)"),
        ExpressionType::list(length())
    );
    assert_eq!(
        admitted_type("members.all(m, m.thickness > quantity(1.0, mm))"),
        ExpressionType::Bool
    );
    assert_eq!(
        admitted_type("let twice = width * 2.0; twice + twice"),
        ExpressionType::Float64
    );
    assert_eq!(denied_family("ready + 1"), Family::TypeMismatch);
    assert_eq!(denied_family("ready ? 1.0 : \"no\""), Family::TypeMismatch);
    assert_eq!(denied_family("frame.weight"), Family::UnknownBinding);
    assert_eq!(denied_family("missing_operand"), Family::UnknownBinding);
}

#[test]
fn casts_are_explicit_and_checked() {
    assert_eq!(
        admitted_type("exact_cast<Int32>(count)"),
        ExpressionType::Integer(IntegerType::Int32)
    );
    assert_eq!(
        admitted_type("rounded_cast<Float32>(width)"),
        ExpressionType::Float32
    );
    assert_eq!(
        denied_family("exact_cast<Int64>(count)"),
        Family::FunctionContractMismatch
    );
    assert_eq!(denied_family("count + small"), Family::TypeMismatch);
    assert_eq!(
        admitted_type("int8(-128)"),
        ExpressionType::Integer(IntegerType::Int8)
    );
    assert_eq!(denied_family("int8(128)"), Family::InvalidValue);
    assert_eq!(denied_family("quantity(1e400, m)"), Family::InvalidValue);
}

#[test]
fn digital_buses_check_widths_statically() {
    assert_eq!(
        admitted_type(r#"bit_and(bus, bits<8>("10100101"))"#),
        ExpressionType::Bits(8)
    );
    assert_eq!(admitted_type("concat(bus, bus)"), ExpressionType::Bits(16));
    assert_eq!(admitted_type("slice<2, 6>(bus)"), ExpressionType::Bits(4));
    assert_eq!(admitted_type("extend<16>(bus)"), ExpressionType::Bits(16));
    assert_eq!(
        admitted_type(r#"enable == logic4("X")"#),
        ExpressionType::Bool
    );
    assert_eq!(
        admitted_type(r#"logic_eq(enable, logic4("1"))"#),
        ExpressionType::Logic4(1)
    );
    assert_ne!(denied_family(r#"bits<8>("0101")"#), Family::Syntax);
    assert_eq!(denied_family("truncate<8>(bus)"), Family::Bounds);
    assert_eq!(denied_family("slice<4, 9>(bus)"), Family::Bounds);
    assert_eq!(
        denied_family(r#"bit_and(bus, bits<4>("1010"))"#),
        Family::FunctionContractMismatch
    );
    assert_eq!(denied_family("extend<5000>(bus)"), Family::ResourceExceeded);
}

#[test]
fn installed_functions_resolve_exact_signatures() {
    let schema = schema();
    let mut builder = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
    let area = function(
        "geometry::area",
        &[
            ("w", ExpressionType::Float64),
            ("h", ExpressionType::Float64),
        ],
        ExpressionType::Float64,
        "w * h",
    );
    builder.install(area.clone()).expect("area installs");
    let duplicate = builder
        .install(area)
        .expect_err("an identical signature is ambiguous");
    assert_eq!(duplicate.family(), Family::AmbiguousBinding);
    let wrong_result = function("geometry::bad", &[], ExpressionType::Bool, "1.0");
    assert_eq!(
        builder.install(wrong_result).unwrap_err().family(),
        Family::TypeMismatch
    );
    let reserved = function("sqrt", &[], ExpressionType::Float64, "1.0");
    assert!(builder.install(reserved).is_err());
    let catalog = builder.build();
    assert_eq!(catalog.functions().len(), 1);

    let admitted = expressions()
        .parse("geometry::area(width, depth) > 1.0")
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .expect("the call admits");
    assert_eq!(admitted.functions().len(), 1);
    let denial = expressions()
        .parse("geometry::area(width, count)")
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .expect_err("no overload takes an Int64");
    assert_eq!(denial.family(), Family::TypeMismatch);
}

#[test]
fn functions_see_only_their_parameters() {
    let schema = schema();
    let mut builder = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
    let leaky = function(
        "geometry::leak",
        &[("w", ExpressionType::Float64)],
        ExpressionType::Float64,
        "w + depth",
    );
    assert_eq!(
        builder.install(leaky).unwrap_err().family(),
        Family::UnknownBinding
    );
}

#[test]
fn expected_result_types_are_enforced() {
    let schema = schema();
    let catalog = super::empty_catalog(&schema);
    let draft = expressions().parse("width * 2.0").expect("parses");
    let denial = draft
        .admit_as(
            &schema,
            &catalog,
            ExpressionProfile::interactive(),
            &ExpressionType::Bool,
        )
        .expect_err("a Float64 is not a Bool");
    assert!(matches!(
        denial.detail(),
        ExpressionDenialDetail::TypeMismatch { .. }
    ));
    let record = ExpressionType::Record(nominal("Frame"));
    let unknown = ExpressionType::Record(nominal("Door"));
    assert!(draft
        .admit_as(
            &schema,
            &catalog,
            ExpressionProfile::interactive(),
            &unknown
        )
        .is_err());
    assert!(draft
        .admit_as(&schema, &catalog, ExpressionProfile::interactive(), &record)
        .is_err());
}
