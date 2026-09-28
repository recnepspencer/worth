//! Exact denial families for builtin, constructor, binder, record, and
//! decoder contracts, plus the admission meter and identity invariants that
//! guard them.

use worth_foundational::expression_api::{
    expressions, Dynamic, ExpressionBuilder, ExpressionDenial, ExpressionDenialDetail,
    ExpressionDenialFamily as Family, ExpressionFunctionCatalog, ExpressionFunctionDeclaration,
    ExpressionProfile, ExpressionResource, ExpressionSchema, ExpressionType,
    ExpressionTypeArgument, ExpressionTypeName, IntegerType, Term,
};

use super::{admit, denied_family, draft, empty_catalog, function, schema};

fn exceeded<T: std::fmt::Debug>(result: Result<T, ExpressionDenial>) -> ExpressionResource {
    let denial = result.expect_err("a resource ceiling denies");
    match denial.detail() {
        ExpressionDenialDetail::ResourceExceeded { resource, .. } => *resource,
        other => panic!("expected a resource denial, found {other:?}"),
    }
}

fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("narrowing lowers a ceiling")
}

fn area() -> ExpressionFunctionDeclaration {
    function(
        "geometry::area",
        &[
            ("w", ExpressionType::Float64),
            ("h", ExpressionType::Float64),
        ],
        ExpressionType::Float64,
        "w * h",
    )
}

fn catalog(
    schema: &ExpressionSchema,
    functions: impl IntoIterator<Item = ExpressionFunctionDeclaration>,
) -> ExpressionFunctionCatalog {
    let mut builder = ExpressionFunctionCatalog::builder(schema, ExpressionProfile::interactive());
    for declaration in functions {
        builder
            .install(declaration)
            .expect("fixture functions install");
    }
    builder.build()
}

#[test]
fn builtins_deny_mismatched_arguments_by_contract() {
    for source in [
        "sqrt(count)",
        "min(width)",
        "clamp(width, count, depth)",
        "near(width, depth)",
        "unwrap(width)",
        "length(width)",
        "starts_with(width, \"a\")",
        "bit_not(count)",
        "case_equal(bus, bus)",
        "logic_eq(bus, bus)",
        "mux(enable, bus, bus)",
    ] {
        assert_eq!(
            denied_family(source),
            Family::FunctionContractMismatch,
            "{source}"
        );
    }
    assert_eq!(denied_family("frobnicate(width)"), Family::UnknownBinding);
    assert_eq!(denied_family("width / clear_width"), Family::TypeMismatch);
    // Width families: invalid type widths, profile ceilings, operation bounds.
    assert_eq!(denied_family("to_bits<0>(count)"), Family::InvalidValue);
    assert_eq!(denied_family("truncate<0>(bus)"), Family::InvalidValue);
    assert_eq!(denied_family("extend<5000>(bus)"), Family::ResourceExceeded);
    assert_eq!(denied_family("truncate<8>(bus)"), Family::Bounds);
}

#[test]
fn type_arguments_are_denied_where_callees_take_none() {
    // Source spells type arguments only on generic intrinsics; builders and
    // decoded drafts can attach them to any call.
    assert_eq!(
        expressions()
            .parse("geometry::area<Int8>(width, depth)")
            .unwrap_err()
            .family(),
        Family::Syntax
    );
    let int8 = ExpressionTypeArgument::Type(ExpressionType::Integer(IntegerType::Int8));
    let schema = schema();
    let catalog = catalog(&schema, [area()]);
    let calls: [(&str, fn(&mut ExpressionBuilder) -> Vec<Term<Dynamic>>); 6] = [
        ("geometry::area", |b| vec![b.name("width"), b.name("depth")]),
        ("int32", |b| vec![b.int(5).dynamic()]),
        ("float64", |b| vec![b.float(1.0).dynamic()]),
        ("decimal", |b| vec![b.text("1.5").dynamic()]),
        ("sqrt", |b| vec![b.name("width")]),
        ("bit_not", |b| vec![b.name("bus")]),
    ];
    for (function, arguments) in calls {
        let mut builder = ExpressionBuilder::new();
        let arguments = arguments(&mut builder);
        let call = builder.generic_call(function, std::slice::from_ref(&int8), &arguments);
        let denial = builder
            .finish(call)
            .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
            .unwrap_err();
        assert_eq!(denial.family(), Family::UnsupportedFeature, "{function}");
    }
    // Every reserved constructor but `bits<N>` denies type arguments.
    let constructors = [
        "int8",
        "int16",
        "int32",
        "int64",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "float32",
        "float64",
        "decimal",
        "bytes",
        "logic4",
        "quantity",
        "magnitude",
    ];
    for function in constructors {
        let mut builder = ExpressionBuilder::new();
        let argument = builder.int(5).dynamic();
        let call = builder.generic_call(function, std::slice::from_ref(&int8), &[argument]);
        let denial = builder
            .finish(call)
            .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
            .unwrap_err();
        assert_eq!(denial.family(), Family::UnsupportedFeature, "{function}");
    }
    assert_eq!(denied_family("INT8(5)"), Family::UnknownBinding);
    assert_eq!(denied_family("Int8(5)"), Family::UnknownBinding);
}

#[test]
fn decimal_constructors_bound_and_normalize() {
    assert_eq!(
        denied_family(r#"decimal("1.0000000000000000001")"#),
        Family::InvalidValue
    );
    let digits = "1".repeat(39);
    assert_eq!(
        denied_family(&format!(r#"decimal("{digits}")"#)),
        Family::InvalidValue
    );
    let identity = |source: &str| admit(source).expect(source).identity().clone();
    assert_eq!(
        identity(r#"decimal("1.50")"#),
        identity(r#"decimal("1.5")"#)
    );
}

#[test]
fn records_and_binders_deny_ambiguity() {
    let steel = "material: Material::Steel";
    assert_eq!(
        denied_family(&format!(
            "Frame {{ thickness: clear_width, thickness: clear_width, {steel} }}"
        )),
        Family::AmbiguousBinding
    );
    assert_eq!(
        denied_family(&format!(
            "Frame {{ thickness: clear_width, {steel}, color: 1 }}"
        )),
        Family::UnknownBinding
    );
    for source in [
        "let width = 1.0; width",
        "let a = 1.0; let a = 2.0; a",
        "members.map(m, members.map(m, m.thickness))",
        "let min = 1.0; min",
    ] {
        assert_eq!(denied_family(source), Family::AmbiguousBinding, "{source}");
    }
    let schema = schema();
    let mut builder = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
    let intrinsic = function(
        "geometry::bad",
        &[("slice", ExpressionType::Float64)],
        ExpressionType::Float64,
        "1.0",
    );
    assert_eq!(
        builder.install(intrinsic).unwrap_err().family(),
        Family::AmbiguousBinding
    );
}

#[test]
fn overload_denials_follow_the_declared_signatures() {
    let schema = schema();
    let admit = |catalog: &ExpressionFunctionCatalog, source: &str| {
        expressions()
            .parse(source)
            .and_then(|draft| draft.admit(&schema, catalog, ExpressionProfile::interactive()))
    };
    // Every same-arity overload agrees on Float64, so the argument is checked
    // against it: one or two overloads report the same family.
    let scaled = function(
        "geometry::area",
        &[("w", ExpressionType::Float64), ("h", ExpressionType::INT64)],
        ExpressionType::Float64,
        "w",
    );
    let one = catalog(&schema, [area()]);
    let two = catalog(&schema, [area(), scaled]);
    for catalog in [&one, &two] {
        let denial = admit(catalog, "geometry::area(count, depth)").unwrap_err();
        assert_eq!(denial.family(), Family::TypeMismatch);
    }
    assert_eq!(
        admit(&one, "geometry::area(width, count)")
            .unwrap_err()
            .family(),
        Family::TypeMismatch
    );
    assert_eq!(
        admit(&two, "geometry::area(width, ready)")
            .unwrap_err()
            .family(),
        Family::FunctionContractMismatch
    );
    assert!(admit(&two, "geometry::area(width, count)").is_ok());
}

#[test]
fn decoding_requires_canonical_post_order() {
    let encoded = draft("width + depth").encode();
    let leaf = |source: &str| draft(source).encode()[12..].to_vec();
    let (width, depth) = (leaf("width"), leaf("depth"));
    assert_eq!(encoded.len(), 12 + width.len() + depth.len() + 10);
    let binary = &encoded[encoded.len() - 10..];
    // The same tree with its leaves stored in the other order.
    let mut swapped = encoded[..12].to_vec();
    swapped.extend_from_slice(&depth);
    swapped.extend_from_slice(&width);
    swapped.extend_from_slice(&binary[..2]);
    swapped.extend_from_slice(&1_u32.to_le_bytes());
    swapped.extend_from_slice(&0_u32.to_le_bytes());
    let denial = expressions().decode(&swapped).unwrap_err();
    assert_eq!(denial.family(), Family::InvalidValue);
    assert!(expressions().decode(&encoded).is_ok());
}

#[test]
fn admission_work_and_canonical_bytes_are_exhausted() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
    let long = format!("[{}width]", "width, ".repeat(200));
    let result = draft(&long).admit(
        &schema,
        &catalog,
        narrowed(ExpressionResource::AdmissionWork, 500),
    );
    assert_eq!(exceeded(result), ExpressionResource::AdmissionWork);
    let text = format!(r#"label ?? "{}""#, "x".repeat(400));
    let result = draft(&text).admit(
        &schema,
        &catalog,
        narrowed(ExpressionResource::CanonicalBytes, 256),
    );
    assert_eq!(exceeded(result), ExpressionResource::CanonicalBytes);
}

#[test]
fn four_valued_values_have_no_implied_equality() {
    let schema = ExpressionSchema::builder()
        .record("Sig", 1, [("line", ExpressionType::Logic4(1))])
        .and_then(|builder| builder.operand("enable", ExpressionType::Logic4(1)))
        .and_then(|builder| {
            builder.operand(
                "sig",
                ExpressionType::Record(ExpressionTypeName::new("Sig", 1)?),
            )
        })
        .expect("fixture schema is valid")
        .build();
    let denied = |source: &str| {
        draft(source)
            .admit(
                &schema,
                &empty_catalog(&schema),
                ExpressionProfile::interactive(),
            )
            .expect_err(source)
    };
    for (source, function) in [
        ("enable == enable", "=="),
        ("enable != enable", "!="),
        ("some(enable) == some(enable)", "=="),
        ("[enable] != [enable]", "!="),
        ("sig == sig", "=="),
        ("contains([enable], enable)", "contains"),
    ] {
        match denied(source).detail() {
            ExpressionDenialDetail::FunctionContractMismatch {
                function: named, ..
            } => {
                assert_eq!(named, function, "{source}");
            }
            other => panic!("{source} denied with {other:?}"),
        }
    }
    let admitted = |source: &str| {
        draft(source).admit(
            &schema,
            &empty_catalog(&schema),
            ExpressionProfile::interactive(),
        )
    };
    assert!(admitted("case_equal(sig.line, enable)").is_ok());
    assert!(admitted("logic_eq(sig.line, enable) == logic_eq(enable, enable)").is_err());
}

#[test]
fn bus_widths_are_checked_through_records_and_callees() {
    let wide = ExpressionSchema::builder()
        .record("Wide", 1, [("bus", ExpressionType::Bits(128))])
        .and_then(|builder| {
            builder.operand(
                "wide",
                ExpressionType::Record(ExpressionTypeName::new("Wide", 1)?),
            )
        })
        .expect("fixture schema is valid")
        .build();
    let result = draft("wide").admit(
        &wide,
        &empty_catalog(&wide),
        narrowed(ExpressionResource::BitWidth, 64),
    );
    assert_eq!(exceeded(result), ExpressionResource::BitWidth);

    let schema = schema();
    let hidden = function(
        "digital::wide",
        &[],
        ExpressionType::Bool,
        r#"slice<0, 1>(to_bits<128>(1)) == bits<1>("1")"#,
    );
    let catalog = catalog(&schema, [hidden]);
    let result = draft("digital::wide()").admit(
        &schema,
        &catalog,
        narrowed(ExpressionResource::BitWidth, 64),
    );
    assert_eq!(exceeded(result), ExpressionResource::BitWidth);
}

#[test]
fn identity_ignores_binder_names_and_unrelated_installs() {
    let identity = |source: &str| admit(source).expect(source).identity().clone();
    assert_eq!(
        identity("let a = width; a + a"),
        identity("let b = width; b + b")
    );
    assert_ne!(identity("extend<16>(bus)"), identity("extend<17>(bus)"));

    let schema = schema();
    let perimeter = function(
        "geometry::perimeter",
        &[("w", ExpressionType::Float64)],
        ExpressionType::Float64,
        "w * 4.0",
    );
    let call = |catalog: &ExpressionFunctionCatalog| {
        draft("geometry::area(width, depth)")
            .admit(&schema, catalog, ExpressionProfile::interactive())
            .expect("the call admits")
            .identity()
            .clone()
    };
    let alone = call(&catalog(&schema, [area()]));
    assert_eq!(alone, call(&catalog(&schema, [area(), perimeter.clone()])));
    assert_eq!(alone, call(&catalog(&schema, [perimeter, area()])));

    let mut bumped = area();
    bumped.name = ExpressionTypeName::new("geometry::area", 2).expect("valid name");
    assert_ne!(alone, call(&catalog(&schema, [bumped])));
}
