//! Consumed reads: an evaluation records exactly the operand paths its
//! outcome depended on, at the weakest kind that decided it, and binding
//! checks every value before evaluation can read it.

use worth_foundational::expression_api::{
    ExpressionDenialFamily as Family, ExpressionInputs, ExpressionPathStep as Step,
    ExpressionProfile, ExpressionReadKind as Kind, ExpressionValue,
};

use super::{admit, evaluate, frame, schema};

/// `(operand, path, kind)` for every recorded read, in order.
fn reads(source: &str) -> Vec<(String, Vec<Step>, Kind)> {
    evaluate(source)
        .consumption()
        .reads()
        .iter()
        .map(|read| {
            (
                read.operand().to_string(),
                read.path().to_vec(),
                read.kind(),
            )
        })
        .collect()
}

fn read(operand: &str, path: &[Step], kind: Kind) -> (String, Vec<Step>, Kind) {
    (operand.to_string(), path.to_vec(), kind)
}

#[test]
fn field_access_records_only_the_field() {
    assert_eq!(
        reads("frame.thickness > quantity(1.0, mm)"),
        [read("frame", &[Step::Field(0)], Kind::Value)]
    );
}

#[test]
fn short_circuits_record_only_what_decided() {
    assert_eq!(
        reads("count > 0 || ready"),
        [read("count", &[], Kind::Value)]
    );
    assert_eq!(
        reads("ready ? 1 : count"),
        [read("ready", &[], Kind::Value)]
    );
    assert_eq!(
        reads(r#"label ?? "none""#),
        [read("label", &[], Kind::Presence)]
    );
    assert_eq!(
        reads("members.any(m, m.material == Material::Steel)"),
        [
            read("members", &[], Kind::Prefix(1)),
            read("members", &[Step::Element(0), Step::Field(1)], Kind::Value),
        ]
    );
}

#[test]
fn filters_record_membership_predicates_and_selected_fields() {
    let element = |index, field| [Step::Element(index), Step::Field(field)];
    assert_eq!(
        reads("members.filter(m, m.material == Material::Steel).map(m, m.thickness)"),
        [
            read("members", &[], Kind::Length),
            read("members", &element(0, 0), Kind::Value),
            read("members", &element(0, 1), Kind::Value),
            read("members", &element(1, 1), Kind::Value),
            read("members", &element(2, 0), Kind::Value),
            read("members", &element(2, 1), Kind::Value),
        ]
    );
}

#[test]
fn collection_builtins_record_length_prefix_or_element() {
    assert_eq!(
        reads("length(members)"),
        [read("members", &[], Kind::Length)]
    );
    assert_eq!(
        reads("get(members, 1)"),
        [
            read("members", &[], Kind::Prefix(2)),
            read("members", &[Step::Element(1)], Kind::Value),
        ]
    );
    assert_eq!(
        reads("get(members, 9)"),
        [read("members", &[], Kind::Length)]
    );
    assert!(reads("get(members, -1)").is_empty());
    assert!(reads("length(bus)").is_empty());
}

#[test]
fn unbound_operands_deny_only_when_read() {
    let inputs = ExpressionInputs::builder(&schema())
        .bind("ready", ExpressionValue::bool(true))
        .expect("binds")
        .build();
    let profile = ExpressionProfile::interactive();
    let run = |source: &str| {
        let compiled = admit(source).expect("admits").compile();
        compiled.evaluate(&inputs, &profile).into_result()
    };
    assert_eq!(run("ready || count > 1"), Ok(ExpressionValue::bool(true)));
    let denial = run("count + 1").expect_err("count is unbound");
    assert_eq!(denial.family(), Family::MissingOperand);
}

#[test]
fn binding_checks_names_and_values() {
    let bind = |name: &str, value: ExpressionValue| {
        ExpressionInputs::builder(&schema())
            .bind(name, value)
            .map(|_| ())
            .map_err(|denial| denial.family())
    };
    assert_eq!(bind("ready", ExpressionValue::bool(true)), Ok(()));
    assert_eq!(
        bind("nope", ExpressionValue::bool(true)),
        Err(Family::UnknownBinding)
    );
    assert_eq!(
        bind("ready", ExpressionValue::integer(1)),
        Err(Family::TypeMismatch)
    );
    assert_eq!(
        bind("small", ExpressionValue::integer(1 << 40)),
        Err(Family::InvalidValue)
    );
    assert_eq!(bind("frame", frame(0.01, 7)), Err(Family::TypeMismatch));
    assert_eq!(
        bind(
            "members",
            ExpressionValue::list(vec![ExpressionValue::bool(true)])
        ),
        Err(Family::TypeMismatch)
    );
    let twice = ExpressionInputs::builder(&schema())
        .bind("ready", ExpressionValue::bool(true))
        .and_then(|builder| builder.bind("ready", ExpressionValue::bool(false)));
    assert_eq!(
        twice.map(|_| ()).map_err(|denial| denial.family()),
        Err(Family::AmbiguousBinding)
    );
}
