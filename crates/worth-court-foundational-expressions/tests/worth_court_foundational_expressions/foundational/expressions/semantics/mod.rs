// Admission semantics: result types, closed operator contracts, and the
// typed denials for width, unit, and type errors.
pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::ExpressionDenialDetail, worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionFunctionCatalog, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionType, worth_foundational::expression_api::IntegerType};

pub(crate) use {crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::admitted_type, crate::expressions::ordinary_context::denied_family, crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::length, crate::expressions::ordinary_context::nominal, crate::expressions::ordinary_context::schema};

#[test]
fn engineering_arithmetic_keeps_types_and_dimensions() {
    assert_eq!(
        admitted_type("width - 2.0 * depth"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    assert_eq!(admitted_type("count + 1"), worth_foundational::expression_api::ExpressionType::INT64);
    assert_eq!(
        admitted_type("quantity(900.0, mm) <= clear_width"),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        admitted_type("clear_width - frame.thickness * 2.0"),
        length()
    );
    assert_eq!(
        admitted_type("magnitude(clear_width, mm)"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
}

#[test]
fn quantities_deny_dimension_and_scalar_mismatches() {
    assert_eq!(
        denied_family("clear_width + quantity(1.0, s)"),
        worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
    );
    assert_eq!(denied_family("clear_width + width"), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
    assert_eq!(
        denied_family("quantity(1.0, parsec)"),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
}

#[test]
fn logic_options_and_collections_type_exactly() {
    assert_eq!(admitted_type("ready && !ready"), worth_foundational::expression_api::ExpressionType::Bool);
    assert_eq!(
        admitted_type(r#"label ?? "Nothing selected""#),
        worth_foundational::expression_api::ExpressionType::String
    );
    assert_eq!(
        admitted_type("members.filter(m, m.material == Material::Steel).map(m, m.thickness)"),
        worth_foundational::expression_api::ExpressionType::list(length())
    );
    assert_eq!(
        admitted_type("members.all(m, m.thickness > quantity(1.0, mm))"),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        admitted_type("let twice = width * 2.0; twice + twice"),
        worth_foundational::expression_api::ExpressionType::Float64
    );
    assert_eq!(denied_family("ready + 1"), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
    assert_eq!(denied_family("ready ? 1.0 : \"no\""), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
    assert_eq!(denied_family("frame.weight"), worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding);
    assert_eq!(denied_family("missing_operand"), worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding);
}

#[test]
fn admitted_programs_report_whether_they_bind_let() {
    let binds_let = |source: &str| {
        admit(source)
            .unwrap_or_else(|denial| panic!("{source} was denied: {denial:?}"))
            .binds_let()
    };
    assert!(binds_let("let twice = width * 2.0; twice + twice"));
    assert!(!binds_let("width * 2.0 + depth"));
    assert!(!binds_let(
        "members.all(m, m.thickness > quantity(1.0, mm))"
    ));
    assert!(binds_let(
        "members.all(m, let thick = m.thickness; thick > quantity(1.0, mm))"
    ));
}

#[test]
fn casts_are_explicit_and_checked() {
    assert_eq!(
        admitted_type("exact_cast<Int32>(count)"),
        worth_foundational::expression_api::ExpressionType::Integer(worth_foundational::expression_api::IntegerType::Int32)
    );
    assert_eq!(
        admitted_type("rounded_cast<Float32>(width)"),
        worth_foundational::expression_api::ExpressionType::Float32
    );
    assert_eq!(
        denied_family("exact_cast<Int64>(count)"),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
    );
    assert_eq!(denied_family("count + small"), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
    assert_eq!(
        admitted_type("int8(-128)"),
        worth_foundational::expression_api::ExpressionType::Integer(worth_foundational::expression_api::IntegerType::Int8)
    );
    assert_eq!(denied_family("int8(128)"), worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue);
    assert_eq!(denied_family("quantity(1e400, m)"), worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue);
}

#[test]
fn digital_buses_check_widths_statically() {
    assert_eq!(
        admitted_type(r#"bit_and(bus, bits<8>("10100101"))"#),
        worth_foundational::expression_api::ExpressionType::Bits(8)
    );
    assert_eq!(admitted_type("concat(bus, bus)"), worth_foundational::expression_api::ExpressionType::Bits(16));
    assert_eq!(admitted_type("slice<2, 6>(bus)"), worth_foundational::expression_api::ExpressionType::Bits(4));
    assert_eq!(admitted_type("extend<16>(bus)"), worth_foundational::expression_api::ExpressionType::Bits(16));
    assert_eq!(
        admitted_type(r#"case_equal(enable, logic4("X"))"#),
        worth_foundational::expression_api::ExpressionType::Bool
    );
    assert_eq!(
        denied_family(r#"enable == logic4("X")"#),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
    );
    assert_eq!(
        admitted_type(r#"logic_eq(enable, logic4("1"))"#),
        worth_foundational::expression_api::ExpressionType::Logic4(1)
    );
    assert_eq!(denied_family(r#"bits<8>("0101")"#), worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue);
    assert_eq!(denied_family("truncate<8>(bus)"), worth_foundational::expression_api::ExpressionDenialFamily::Bounds);
    assert_eq!(denied_family("slice<4, 9>(bus)"), worth_foundational::expression_api::ExpressionDenialFamily::Bounds);
    assert_eq!(
        denied_family(r#"bit_and(bus, bits<4>("1010"))"#),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
    );
    assert_eq!(denied_family("extend<5000>(bus)"), worth_foundational::expression_api::ExpressionDenialFamily::ResourceExceeded);
}

#[test]
fn installed_functions_resolve_exact_signatures() {
    let schema = schema();
    let mut builder = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(&schema, worth_foundational::expression_api::ExpressionProfile::interactive());
    let area = function(
        "geometry::area",
        &[
            ("w", worth_foundational::expression_api::ExpressionType::Float64),
            ("h", worth_foundational::expression_api::ExpressionType::Float64),
        ],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w * h",
    );
    builder.install(area.clone()).expect("area installs");
    let duplicate = builder
        .install(area)
        .expect_err("an identical signature is ambiguous");
    assert_eq!(duplicate.family(), worth_foundational::expression_api::ExpressionDenialFamily::AmbiguousBinding);
    let wrong_result = function("geometry::bad", &[], worth_foundational::expression_api::ExpressionType::Bool, "1.0");
    assert_eq!(
        builder.install(wrong_result).unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
    );
    let reserved = function("sqrt", &[], worth_foundational::expression_api::ExpressionType::Float64, "1.0");
    assert_eq!(
        builder.install(reserved).unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::UnsupportedFeature
    );
    let catalog = builder.build();
    assert_eq!(catalog.functions().len(), 1);

    let admitted = expressions()
        .parse("geometry::area(width, depth) > 1.0")
        .and_then(|draft| draft.admit(&schema, &catalog, worth_foundational::expression_api::ExpressionProfile::interactive()))
        .expect("the call admits");
    assert_eq!(admitted.functions().len(), 1);
    let denial = expressions()
        .parse("geometry::area(width, count)")
        .and_then(|draft| draft.admit(&schema, &catalog, worth_foundational::expression_api::ExpressionProfile::interactive()))
        .expect_err("no overload takes an Int64");
    assert_eq!(denial.family(), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
}

#[test]
fn functions_see_only_their_parameters() {
    let schema = schema();
    let mut builder = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(&schema, worth_foundational::expression_api::ExpressionProfile::interactive());
    let leaky = function(
        "geometry::leak",
        &[("w", worth_foundational::expression_api::ExpressionType::Float64)],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w + depth",
    );
    assert_eq!(
        builder.install(leaky).unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
}

#[test]
fn expected_result_types_are_enforced() {
    let schema = schema();
    let catalog = crate::expressions::ordinary_context::empty_catalog(&schema);
    let draft = expressions().parse("width * 2.0").expect("parses");
    let denial = draft
        .admit_as(
            &schema,
            &catalog,
            worth_foundational::expression_api::ExpressionProfile::interactive(),
            &worth_foundational::expression_api::ExpressionType::Bool,
        )
        .expect_err("a Float64 is not a Bool");
    assert!(matches!(
        denial.detail(),
        worth_foundational::expression_api::ExpressionDenialDetail::TypeMismatch { .. }
    ));
    let record = worth_foundational::expression_api::ExpressionType::Record(nominal("Frame"));
    let unknown = worth_foundational::expression_api::ExpressionType::Record(nominal("Door"));
    let family = |expected: &ExpressionType| {
        draft
            .admit_as(
                &schema,
                &catalog,
                worth_foundational::expression_api::ExpressionProfile::interactive(),
                expected,
            )
            .unwrap_err()
            .family()
    };
    assert_eq!(family(&unknown), worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding);
    assert_eq!(family(&record), worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch);
}
