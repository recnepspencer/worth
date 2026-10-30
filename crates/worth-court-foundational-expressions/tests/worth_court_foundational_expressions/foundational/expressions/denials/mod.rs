mod admission_limits;
mod identity_equivalence;
mod type_argument_contracts;

// Exact denial families for builtin, constructor, binder, record, and
// decoder contracts, plus the admission meter and identity invariants that
// guard them.
#[allow(unused_imports)]
use crate::expressions::ordinary_context::denials::*;
pub(crate) use {
    worth_foundational::expression_api::expressions, worth_foundational::expression_api::Dynamic,
    worth_foundational::expression_api::ExpressionBuilder,
    worth_foundational::expression_api::ExpressionDenial,
    worth_foundational::expression_api::ExpressionDenialDetail,
    worth_foundational::expression_api::ExpressionDenialFamily as Family,
    worth_foundational::expression_api::ExpressionFunctionCatalog,
    worth_foundational::expression_api::ExpressionFunctionDeclaration,
    worth_foundational::expression_api::ExpressionProfile,
    worth_foundational::expression_api::ExpressionResource,
    worth_foundational::expression_api::ExpressionSchema,
    worth_foundational::expression_api::ExpressionType,
    worth_foundational::expression_api::ExpressionTypeArgument,
    worth_foundational::expression_api::ExpressionTypeName,
    worth_foundational::expression_api::IntegerType, worth_foundational::expression_api::Term,
};

pub(crate) use {
    crate::expressions::ordinary_context::admit,
    crate::expressions::ordinary_context::denied_family,
    crate::expressions::ordinary_context::draft,
    crate::expressions::ordinary_context::empty_catalog,
    crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::schema,
};

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
            worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch,
            "{source}"
        );
    }
    assert_eq!(
        denied_family("frobnicate(width)"),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
    assert_eq!(
        denied_family("width / clear_width"),
        worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
    );
    // Width families: invalid type widths, profile ceilings, operation bounds.
    assert_eq!(
        denied_family("to_bits<0>(count)"),
        worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue
    );
    assert_eq!(
        denied_family("truncate<0>(bus)"),
        worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue
    );
    assert_eq!(
        denied_family("extend<5000>(bus)"),
        worth_foundational::expression_api::ExpressionDenialFamily::ResourceExceeded
    );
    assert_eq!(
        denied_family("truncate<8>(bus)"),
        worth_foundational::expression_api::ExpressionDenialFamily::Bounds
    );
}

#[test]
fn type_arguments_are_denied_where_callees_take_none() {
    type_argument_contracts::nongeneric_callees_deny_type_arguments();
}

#[test]
fn decimal_constructors_bound_and_normalize() {
    assert_eq!(
        denied_family(r#"decimal("1.0000000000000000001")"#),
        worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue
    );
    let digits = "1".repeat(39);
    assert_eq!(
        denied_family(&format!(r#"decimal("{digits}")"#)),
        worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue
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
        worth_foundational::expression_api::ExpressionDenialFamily::AmbiguousBinding
    );
    assert_eq!(
        denied_family(&format!(
            "Frame {{ thickness: clear_width, {steel}, color: 1 }}"
        )),
        worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding
    );
    for source in [
        "let width = 1.0; width",
        "let a = 1.0; let a = 2.0; a",
        "members.map(m, members.map(m, m.thickness))",
        "let min = 1.0; min",
    ] {
        assert_eq!(
            denied_family(source),
            worth_foundational::expression_api::ExpressionDenialFamily::AmbiguousBinding,
            "{source}"
        );
    }
    let schema = schema();
    let mut builder = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(
        &schema,
        worth_foundational::expression_api::ExpressionProfile::interactive(),
    );
    let intrinsic = function(
        "geometry::bad",
        &[(
            "slice",
            worth_foundational::expression_api::ExpressionType::Float64,
        )],
        worth_foundational::expression_api::ExpressionType::Float64,
        "1.0",
    );
    assert_eq!(
        builder.install(intrinsic).unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::AmbiguousBinding
    );
}

#[test]
fn overload_denials_follow_the_declared_signatures() {
    let schema = schema();
    let admit = |catalog: &ExpressionFunctionCatalog, source: &str| {
        expressions().parse(source).and_then(|draft| {
            draft.admit(
                &schema,
                catalog,
                worth_foundational::expression_api::ExpressionProfile::interactive(),
            )
        })
    };
    // Every same-arity overload agrees on Float64, so the argument is checked
    // against it: one or two overloads report the same family.
    let scaled = function(
        "geometry::area",
        &[
            (
                "w",
                worth_foundational::expression_api::ExpressionType::Float64,
            ),
            (
                "h",
                worth_foundational::expression_api::ExpressionType::INT64,
            ),
        ],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w",
    );
    let one = catalog(&schema, [area()]);
    let two = catalog(&schema, [area(), scaled]);
    for catalog in [&one, &two] {
        let denial = admit(catalog, "geometry::area(count, depth)").unwrap_err();
        assert_eq!(
            denial.family(),
            worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
        );
    }
    assert_eq!(
        admit(&one, "geometry::area(width, count)")
            .unwrap_err()
            .family(),
        worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch
    );
    assert_eq!(
        admit(&two, "geometry::area(width, ready)")
            .unwrap_err()
            .family(),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
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
    assert_eq!(
        denial.family(),
        worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue
    );
    assert!(expressions().decode(&encoded).is_ok());
}

#[test]
fn admission_work_and_canonical_bytes_are_exhausted() {
    admission_limits::work_and_canonical_byte_limits_are_enforced();
}

#[test]
fn four_valued_values_have_no_implied_equality() {
    let schema = worth_foundational::expression_api::ExpressionSchema::builder()
        .record(
            "Sig",
            1,
            [(
                "line",
                worth_foundational::expression_api::ExpressionType::Logic4(1),
            )],
        )
        .and_then(|builder| {
            builder.operand(
                "enable",
                worth_foundational::expression_api::ExpressionType::Logic4(1),
            )
        })
        .and_then(|builder| {
            builder.operand(
                "sig",
                worth_foundational::expression_api::ExpressionType::Record(
                    worth_foundational::expression_api::ExpressionTypeName::new("Sig", 1)?,
                ),
            )
        })
        .expect("fixture schema is valid")
        .build();
    let denied = |source: &str| {
        draft(source)
            .admit(
                &schema,
                &empty_catalog(&schema),
                worth_foundational::expression_api::ExpressionProfile::interactive(),
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
            worth_foundational::expression_api::ExpressionDenialDetail::FunctionContractMismatch {
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
            worth_foundational::expression_api::ExpressionProfile::interactive(),
        )
    };
    assert!(admitted("case_equal(sig.line, enable)").is_ok());
    let result = admitted("logic_eq(sig.line, enable) == logic_eq(enable, enable)");
    assert_eq!(
        result.unwrap_err().family(),
        worth_foundational::expression_api::ExpressionDenialFamily::FunctionContractMismatch
    );
}

#[test]
fn bus_widths_are_checked_through_records_and_callees() {
    admission_limits::record_and_callee_bus_widths_are_bounded();
}

#[test]
fn identity_ignores_binder_names_and_unrelated_installs() {
    identity_equivalence::binder_and_catalog_changes_preserve_expected_identity();
}
