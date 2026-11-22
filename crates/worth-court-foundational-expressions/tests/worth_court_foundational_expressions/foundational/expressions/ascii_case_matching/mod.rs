// ASCII case-insensitive containment and affixes: exact over ASCII-folded
// text, String-only, and metered like their exact counterparts one 8-byte
// word at a time, so suspension and resumption mid-match change nothing.
use proptest::prelude::*;
use worth_foundational::expression_api::{
    expressions, AdmittedExpression, ExpressionFunctionCatalog, ExpressionInputs,
    ExpressionProfile, ExpressionResource, ExpressionSchema, ExpressionStep, ExpressionType,
    ExpressionValue,
};

fn schema() -> ExpressionSchema {
    ExpressionSchema::builder()
        .operand("text", ExpressionType::String)
        .and_then(|builder| builder.operand("probe", ExpressionType::String))
        .and_then(|builder| builder.operand("raw", ExpressionType::Bytes))
        .expect("fixture operands are valid")
        .build()
}

fn admit(source: &str) -> AdmittedExpression {
    let schema = schema();
    let catalog =
        ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    expressions()
        .parse(source)
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .unwrap_or_else(|denial| panic!("{source}: {denial:?}"))
}

fn inputs(text: &str, probe: &str) -> ExpressionInputs {
    ExpressionInputs::builder(&schema())
        .bind("text", ExpressionValue::string(text))
        .and_then(|builder| builder.bind("probe", ExpressionValue::string(probe)))
        .and_then(|builder| builder.bind("raw", ExpressionValue::bytes(text.as_bytes().to_vec())))
        .expect("fixture inputs conform")
        .build()
}

/// `(result, semantic work, slices)` of a run sliced at `quantum`.
fn stepped(source: &str, text: &str, probe: &str, quantum: u64) -> (bool, u64, u64) {
    let mut continuation = admit(source)
        .compile()
        .start(&inputs(text, probe), &ExpressionProfile::interactive())
        .expect("inputs bind");
    loop {
        match continuation.step(quantum) {
            ExpressionStep::Complete(evaluation) => {
                let cost = evaluation.cost().clone();
                let value = evaluation.into_result().expect("evaluates");
                return (
                    value.as_bool().expect("Bool result"),
                    cost.used(ExpressionResource::SemanticWork),
                    cost.slices(),
                );
            }
            ExpressionStep::Suspended(next) => continuation = next,
        }
    }
}

const OPERATIONS: [(&str, &str); 3] = [
    (
        "contains_ignore_ascii_case(text, probe)",
        "contains(text, probe)",
    ),
    (
        "starts_with_ignore_ascii_case(text, probe)",
        "starts_with(text, probe)",
    ),
    (
        "ends_with_ignore_ascii_case(text, probe)",
        "ends_with(text, probe)",
    ),
];

fn folded_oracle(operation: usize, text: &str, probe: &str) -> bool {
    let (text, probe) = (text.to_ascii_lowercase(), probe.to_ascii_lowercase());
    match operation {
        0 => text.contains(&probe),
        1 => text.starts_with(&probe),
        _ => text.ends_with(&probe),
    }
}

#[test]
fn folding_is_ascii_only_and_string_only() {
    let cases = [
        ("Stainless STEEL", "steel", [true, false, true]),
        ("Café CRÈME", "café crÈme", [true, true, true]),
        ("ÉCOLE", "école", [false, false, false]),
        ("Straße", "STRASSE", [false, false, false]),
        ("", "", [true, true, true]),
        ("abc", "ABCD", [false, false, false]),
    ];
    for (text, probe, expected) in cases {
        for ((folded, _), expected) in OPERATIONS.iter().zip(expected) {
            assert_eq!(
                stepped(folded, text, probe, u64::MAX).0,
                expected,
                "{folded} {text:?} {probe:?}"
            );
        }
    }
    let schema = schema();
    let catalog =
        ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    for source in [
        "contains_ignore_ascii_case(raw, raw)",
        "starts_with_ignore_ascii_case(text, raw)",
        "ends_with_ignore_ascii_case(text)",
    ] {
        let denied = expressions()
            .parse(source)
            .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()));
        assert!(denied.is_err(), "{source} must be denied");
    }
}

#[test]
fn a_long_match_suspends_and_resumes_mid_scan_without_changing_its_bill() {
    let text = format!("{}NEEDLE-in-HAYSTACK", "x".repeat(200));
    for (folded, exact) in OPERATIONS {
        let whole = stepped(folded, &text, "needle-IN-haystack", u64::MAX);
        let sliced = stepped(folded, &text, "needle-IN-haystack", 1);
        assert_eq!((sliced.0, sliced.1), (whole.0, whole.1), "{folded}");
        assert!(sliced.2 > 1, "{folded} suspended at least once");
        let lowered = stepped(
            exact,
            &text.to_ascii_lowercase(),
            "needle-in-haystack",
            u64::MAX,
        );
        assert_eq!(
            (whole.0, whole.1),
            (lowered.0, lowered.1),
            "{folded} bills like {exact}"
        );
    }
}

fn mixed_case_text() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[aAbBzZ0_é]{0,40}").expect("valid regex")
}

proptest! {
    #[test]
    fn folded_matching_equals_exact_matching_over_folded_text_at_equal_cost(
        text in mixed_case_text(),
        probe in mixed_case_text().prop_map(|probe| probe.chars().take(12).collect::<String>()),
        quantum in 0_u64..6,
    ) {
        let (lower_text, lower_probe) = (text.to_ascii_lowercase(), probe.to_ascii_lowercase());
        for (operation, (folded, exact)) in OPERATIONS.iter().enumerate() {
            let whole = stepped(folded, &text, &probe, u64::MAX);
            let sliced = stepped(folded, &text, &probe, quantum);
            let lowered = stepped(exact, &lower_text, &lower_probe, u64::MAX);
            prop_assert_eq!(whole.0, folded_oracle(operation, &text, &probe), "{} {:?} {:?}", folded, &text, &probe);
            prop_assert_eq!((sliced.0, sliced.1), (whole.0, whole.1));
            prop_assert_eq!((whole.0, whole.1), (lowered.0, lowered.1));
        }
    }
}
