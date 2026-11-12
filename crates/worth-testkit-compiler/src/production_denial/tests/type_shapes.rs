//! Parsed semantic type pairs and arity, never rendered compiler source.
use super::*;

fn mismatch(pairs: &[(&str, &str)]) -> Value {
    let mut outcome = observation(&["mismatched"]);
    let error = &mut outcome["messages"][0];
    error["code"]["code"] = "E0308".into();
    error["message"] = "mismatched types".into();
    error["children"] = Value::Array(
        pairs
            .iter()
            .map(|(expected, found)| {
                json!({
                    "level":"note", "code":null, "rendered":null, "spans":[], "children":[],
                    "message":format!("expected struct `{expected}`\nfound struct `{found}`")
                })
            })
            .collect(),
    );
    outcome
}

fn check_pairs(
    outcome: &Value,
    pairs: &[ProductionTypeMismatchPair<'_>],
) -> Result<(), CompilerFixtureRefusal> {
    check_production_outcome(
        &source(),
        outcome,
        &[ProductionDiagnosticExpectation::TypeMismatch {
            pairs,
            symbols: &["mismatched"],
        }],
    )
}

#[test]
fn required_and_supplied_type_roles_cannot_be_reversed_or_cross_joined() {
    let pairs = [
        ProductionTypeMismatchPair {
            expected: "Projection<Kind>",
            found: "&str",
        },
        ProductionTypeMismatchPair {
            expected: "Digest<Authority, Kind>",
            found: "&str",
        },
    ];
    assert!(check_pairs(
        &mismatch(&[
            ("Projection<Kind>", "&str"),
            ("Digest<Authority, Kind>", "&str")
        ]),
        &pairs
    )
    .is_ok());
    for actual in [
        vec![
            ("&str", "Projection<Kind>"),
            ("&str", "Digest<Authority, Kind>"),
        ],
        vec![
            ("Projection<Kind>", "&str"),
            ("Digest<Authority, Kind>", "Other"),
        ],
    ] {
        assert!(check_pairs(&mismatch(&actual), &pairs).is_err());
    }
}

#[test]
fn repeated_type_mismatches_need_distinct_canonical_notes_not_duplicate_labels() {
    let pairs = [
        ProductionTypeMismatchPair {
            expected: "&Sealed",
            found: "&&Draft",
        },
        ProductionTypeMismatchPair {
            expected: "&Sealed",
            found: "&&Draft",
        },
    ];
    let mut two = mismatch(&[("&Sealed", "&&Draft"), ("&Sealed", "&&Draft")]);
    two["messages"][0]["spans"][0]["label"] = "expected `&Sealed`, found `&&Draft`".into();
    assert!(check_pairs(&two, &pairs).is_ok());
    two["messages"][0]["children"].as_array_mut().unwrap().pop();
    assert!(check_pairs(&two, &pairs).is_err());
}

#[test]
fn type_pair_matching_keeps_reference_unit_and_fully_qualified_identity() {
    for (required, found, actual) in [
        ("Arc<str>", "&str", "String"),
        (
            "Result<(), Failure>",
            "Result<Report, Failure>",
            "Result<(), Failure>",
        ),
        ("owner_one::Identity<Tag>", "&str", "&str"),
    ] {
        let expected_actual = if required.starts_with("owner_one") {
            "owner_two::Identity<Tag>"
        } else {
            required
        };
        let pairs = [ProductionTypeMismatchPair {
            expected: required,
            found,
        }];
        assert!(check_pairs(&mismatch(&[(expected_actual, actual)]), &pairs).is_err());
    }
    assert!(check_pairs(
        &mismatch(&[("Identity<Kind>", "&str")]),
        &[ProductionTypeMismatchPair {
            expected: "owner::Identity<scope::Kind>",
            found: "&str",
        }]
    )
    .is_ok());
}

#[test]
fn argument_count_denials_preserve_required_and_supplied_counts_and_refuse_ambiguity() {
    for (message, required, supplied) in [
        (
            "this method takes 2 arguments but 1 argument was supplied",
            2,
            1,
        ),
        (
            "this function takes 5 arguments but 4 arguments were supplied",
            5,
            4,
        ),
    ] {
        let mut outcome = observation(&["OwnerIdentity"]);
        outcome["messages"][0]["code"]["code"] = "E0061".into();
        outcome["messages"][0]["message"] = message.into();
        outcome["messages"][0]["spans"][0]["label"] = "OwnerIdentity is missing".into();
        let expected = ProductionDiagnosticExpectation::Arity {
            required,
            supplied,
            symbols: &["OwnerIdentity"],
        };
        assert!(check_production_outcome(&source(), &outcome, &[expected]).is_ok());
        for changed in [
            message.replace(" takes ", " takes 7 or "),
            format!("{message} or 9"),
        ] {
            outcome["messages"][0]["message"] = changed.into();
            assert!(check_production_outcome(&source(), &outcome, &[expected]).is_err());
        }
        outcome["messages"][0]["message"] = message.into();
        assert!(check_production_outcome(
            &source(),
            &outcome,
            &[ProductionDiagnosticExpectation::Arity {
                required: supplied,
                supplied: required,
                symbols: &["OwnerIdentity"],
            }]
        )
        .is_err());
    }
}

#[test]
fn empty_type_pairs_and_equal_arity_are_invalid_expectations() {
    for expectation in [
        ProductionDiagnosticExpectation::TypeMismatch {
            pairs: &[],
            symbols: &["mismatched"],
        },
        ProductionDiagnosticExpectation::Arity {
            required: 2,
            supplied: 2,
            symbols: &["mismatched"],
        },
    ] {
        assert!(matches!(
            check_production_outcome(
                &source(),
                &mismatch(&[("Expected", "Found")]),
                &[expectation]
            ),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
}
