// Consumed reads: an evaluation records exactly the operand paths its
// outcome depended on, at the weakest kind that decided it, and binding
// checks every value before evaluation can read it.
#[allow(unused_imports)]
use crate::expressions::ordinary_context::consumption::*;
pub(crate) use worth_foundational::expression_api::ExpressionValue;

pub(crate) use {
    crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::frame,
    crate::expressions::ordinary_context::schema,
};

#[test]
fn field_access_records_only_the_field() {
    assert_eq!(
        reads("frame.thickness > quantity(1.0, mm)"),
        [read(
            "frame",
            &[worth_foundational::expression_api::ExpressionPathStep::Field(0)],
            worth_foundational::expression_api::ExpressionReadKind::Value
        )]
    );
}

#[test]
fn short_circuits_record_only_what_decided() {
    assert_eq!(
        reads("count > 0 || ready"),
        [read(
            "count",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Value
        )]
    );
    assert_eq!(
        reads("ready ? 1 : count"),
        [read(
            "ready",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Value
        )]
    );
    assert_eq!(
        reads(r#"label ?? "none""#),
        [read(
            "label",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Presence
        )]
    );
    assert_eq!(
        reads("members.any(m, m.material == Material::Steel)"),
        [
            read(
                "members",
                &[],
                worth_foundational::expression_api::ExpressionReadKind::Prefix(1)
            ),
            read(
                "members",
                &[
                    worth_foundational::expression_api::ExpressionPathStep::Element(0),
                    worth_foundational::expression_api::ExpressionPathStep::Field(1)
                ],
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
        ]
    );
}

#[test]
fn filters_record_membership_predicates_and_selected_fields() {
    let element = |index, field| {
        [
            worth_foundational::expression_api::ExpressionPathStep::Element(index),
            worth_foundational::expression_api::ExpressionPathStep::Field(field),
        ]
    };
    assert_eq!(
        reads("members.filter(m, m.material == Material::Steel).map(m, m.thickness)"),
        [
            read(
                "members",
                &[],
                worth_foundational::expression_api::ExpressionReadKind::Length
            ),
            read(
                "members",
                &element(0, 0),
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
            read(
                "members",
                &element(0, 1),
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
            read(
                "members",
                &element(1, 1),
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
            read(
                "members",
                &element(2, 0),
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
            read(
                "members",
                &element(2, 1),
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
        ]
    );
}

#[test]
fn collection_builtins_record_length_prefix_or_element() {
    assert_eq!(
        reads("length(members)"),
        [read(
            "members",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Length
        )]
    );
    assert_eq!(
        reads("get(members, 1)"),
        [
            read(
                "members",
                &[],
                worth_foundational::expression_api::ExpressionReadKind::Prefix(2)
            ),
            read(
                "members",
                &[worth_foundational::expression_api::ExpressionPathStep::Element(1)],
                worth_foundational::expression_api::ExpressionReadKind::Value
            ),
        ]
    );
    assert_eq!(
        reads("get(members, 9)"),
        [read(
            "members",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Length
        )]
    );
    assert!(reads("get(members, -1)").is_empty());
    assert!(reads("length(bus)").is_empty());
}

#[test]
fn constructed_maps_keep_each_value_origin() {
    assert_eq!(
        reads(r#"get({"w": width, "d": depth}, "w")"#),
        [read(
            "width",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Value
        )]
    );
    assert_eq!(
        reads(r#"entries({"w": width, "d": depth}).map(e, e.key)"#),
        Vec::new()
    );
    // Keys decide a map's shape, so building it reads every key whole.
    assert_eq!(
        reads("contains({count: width}, 7)"),
        [read(
            "count",
            &[],
            worth_foundational::expression_api::ExpressionReadKind::Value
        )]
    );
}

#[test]
fn unbound_operands_deny_only_when_read() {
    let inputs = worth_foundational::expression_api::ExpressionInputs::builder(&schema())
        .bind(
            "ready",
            worth_foundational::expression_api::ExpressionValue::bool(true),
        )
        .expect("binds")
        .build();
    let profile = worth_foundational::expression_api::ExpressionProfile::interactive();
    let run = |source: &str| {
        let compiled = admit(source).expect("admits").compile();
        compiled.evaluate(&inputs, &profile).into_result()
    };
    assert_eq!(
        run("ready || count > 1"),
        Ok(worth_foundational::expression_api::ExpressionValue::bool(
            true
        ))
    );
    let denial = run("count + 1").expect_err("count is unbound");
    assert_eq!(
        denial.family(),
        worth_foundational::expression_api::ExpressionDenialFamily::MissingOperand
    );
}

#[test]
fn binding_checks_names_and_values() {
    let bind = |name: &str, value: ExpressionValue| {
        worth_foundational::expression_api::ExpressionInputs::builder(&schema())
            .bind(name, value)
            .map(|_| ())
            .map_err(|denial| denial.family())
    };
    assert_eq!(
        bind(
            "ready",
            worth_foundational::expression_api::ExpressionValue::bool(true)
        ),
        Ok(())
    );
    assert_eq!(
        bind(
            "nope",
            worth_foundational::expression_api::ExpressionValue::bool(true)
        ),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::UnknownBinding)
    );
    assert_eq!(
        bind(
            "ready",
            worth_foundational::expression_api::ExpressionValue::integer(1)
        ),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch)
    );
    assert_eq!(
        bind(
            "small",
            worth_foundational::expression_api::ExpressionValue::integer(1 << 40)
        ),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue)
    );
    assert_eq!(
        bind("frame", frame(0.01, 7)),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch)
    );
    assert_eq!(
        bind(
            "members",
            worth_foundational::expression_api::ExpressionValue::list(vec![
                worth_foundational::expression_api::ExpressionValue::bool(true)
            ])
        ),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::TypeMismatch)
    );
    let twice = worth_foundational::expression_api::ExpressionInputs::builder(&schema())
        .bind(
            "ready",
            worth_foundational::expression_api::ExpressionValue::bool(true),
        )
        .and_then(|builder| {
            builder.bind(
                "ready",
                worth_foundational::expression_api::ExpressionValue::bool(false),
            )
        });
    assert_eq!(
        twice.map(|_| ()).map_err(|denial| denial.family()),
        Err(worth_foundational::expression_api::ExpressionDenialFamily::AmbiguousBinding)
    );
}
