//! Malformed corrections are refused and recorded-inverse bindings expand cleanly.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the correction is refused")
        .to_string()
}

const BASE: &str = "schema = Schema, source = SourceBinding, inverse = InverseIntent, \
     disclosure = DisclosureBinding, derive = derive_inverse";

#[test]
fn malformed_corrections_are_refused() {
    let cases = [
        (
            format!("#[worth_query({BASE}, demand = preimage_demand)] struct Fix;"),
            "requires `namespace = NAMESPACE` or `identity = \"..\"`",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, identity = \"a.b\", demand = preimage_demand)] \
                 struct Fix;"
            ),
            "either `namespace = ..` or `identity = \"..\"`, not both",
        ),
        (
            format!("#[worth_query({BASE}, namespace = NS)] struct Fix;"),
            "requires `demand = fn` or `preimage_fields(..), preimage_bytes = ..`",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, demand = preimage_demand, \
                 preimage_fields(TaskFacts::done()), preimage_bytes = 8)] struct Fix;"
            ),
            "not both",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, preimage_fields(), preimage_bytes = 8)] \
                 struct Fix;"
            ),
            "must list at least one field selection",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, preimage_fields(TaskFacts::done()))] \
                 struct Fix;"
            ),
            "requires `#[worth_query(preimage_bytes = ..)]`",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, demand = preimage_demand, bogus = 1)] \
                 struct Fix;"
            ),
            "unknown key for #[derive(WorthQueryCorrection)]",
        ),
        (
            format!(
                "#[worth_query({BASE}, namespace = NS, demand = preimage_demand)] \
                 struct Fix {{ x: u64 }}"
            ),
            "use a unit struct",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

#[test]
fn recorded_inverse_correction_expands_source_body_preimage_and_dispatch() {
    let input = syn::parse_str(
        "#[worth_query(schema = Schema, namespace = CORRECTION, source = CompleteBinding, \
         inverse = RestoreCompletion, disclosure = DetailsBinding, source_body = complete_body, \
         preimage_fields(TaskFacts::done()), preimage_bytes = 8, derive = derive_restore)] \
         pub struct CorrectCompletion;",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    for needle in [
        "impl :: worth_query_decl :: facade :: application_aftermath :: ApplicationCorrectionBinding < Schema > for CorrectCompletion",
        "worth_query_identity ! (CORRECTION , binding)",
        "fn source_body ()",
        "Some (complete_body ())",
        "DeclaredPreImageLocus :: from_field (TaskFacts :: done ())",
        "RecordedInverse (__worth_correction_preimage)",
        "UnsupportedSourceMeaning",
        "derive_restore (__worth_correction_original , __worth_correction_preimage)",
        "__worth_correction_dispatch . conventional ()",
    ] {
        assert!(expanded.contains(needle), "{needle}: {expanded}");
    }

    let with_cap = syn::parse_str(
        "#[worth_query(schema = Schema, identity = \"app.fix.v1\", source = CompleteBinding, \
         inverse = RestoreCompletion, disclosure = DetailsBinding, demand = build_demand, \
         derive = preimage, disclosure_parameters = original, inverse_key = key, capability = AdminCap)] \
         pub struct AdminCorrection;",
    )
    .unwrap();
    let expanded = expand(&with_cap).unwrap().to_string();
    assert!(expanded.contains("\"app.fix.v1\""), "{expanded}");
    assert!(
        expanded.contains("original (__worth_correction_original)"),
        "{expanded}"
    );
    assert!(
        expanded.contains("preimage (__worth_correction_original , __worth_correction_preimage)"),
        "{expanded}"
    );
    assert!(
        expanded.contains("key (__worth_correction_key)"),
        "{expanded}"
    );
    assert!(
        expanded.contains("__worth_correction_dispatch . capability :: < AdminCap > ()"),
        "{expanded}"
    );
    assert!(!expanded.contains("fn source_body"), "{expanded}");
}
