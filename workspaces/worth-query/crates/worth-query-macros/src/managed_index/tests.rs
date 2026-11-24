//! Grammar and hygiene unit tests for `#[derive(WorthManagedIndex)]`.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("test input parses");
    expand(&input)
        .expect_err("input must be refused")
        .to_string()
}

#[test]
fn malformed_managed_indexes_are_refused() {
    let cases = [
        (
            "#[worth_query(schema = S, entity = E, source = Src)] struct MissingProbe;",
            "requires `probe = ValueType` or `probe_binding = BindingType`",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P, probe_binding = PB)] \
             struct BothProbes;",
            "specify either `probe = ValueType` or `probe_binding = BindingType`, not both",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P, provider_version = 0)] \
             struct ZeroVer;",
            "`provider_version` must be at least 1",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P, name = \"\")] \
             struct EmptyName;",
            "`name` must be non-empty",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P)] \
             struct NotUnit { x: i64 }",
            "declares a marker; use a unit struct",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P)] \
             struct Generic<T>;",
            "does not support generic parameters",
        ),
        (
            "#[worth_query(schema = S, entity = E, source = Src, probe = P, bad = 1)] \
             struct UnknownKey;",
            "unknown key for #[derive(WorthManagedIndex)]",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

#[test]
fn valid_managed_index_expands_with_checked_tree_defaults_and_overrides() {
    let input = syn::parse_str(
        "#[worth_query(schema = Schema, entity = Equipment, source = EquipmentSource, probe = QueryBox2D)] \
         pub struct EquipmentBoxIndex;",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(
        expanded.contains("ApplicationManagedIndexBinding"),
        "{expanded}"
    );
    assert!(
        expanded.contains("WorthQueryDefaultValueBinding"),
        "{expanded}"
    );
    assert!(expanded.contains("\"equipment_box_index\""), "{expanded}");
    assert!(
        expanded.contains("\"worth.managed_index.equipment_box_index.v1\""),
        "{expanded}"
    );
    assert!(expanded.contains("BinarySourceTree"), "{expanded}");
    assert!(expanded.contains("CheckedSummarySourceTree"), "{expanded}");
    assert!(expanded.contains("AdmitRebuild"), "{expanded}");

    let explicit = syn::parse_str(
        "#[worth_query(schema = Schema, entity = Task, source = Source, \
         probe_binding = StringApplicationValueBinding, name = \"titles\", \
         provider = \"custom.titles.v2\", provider_version = 2, \
         structure = Opaque, membership = Candidates, maintenance = WhenRequired)] \
         pub struct TitleIdx;",
    )
    .unwrap();
    let expanded = expand(&explicit).unwrap().to_string();
    assert!(
        expanded.contains("StringApplicationValueBinding"),
        "{expanded}"
    );
    assert!(expanded.contains("\"titles\""), "{expanded}");
    assert!(expanded.contains("\"custom.titles.v2\""), "{expanded}");
    assert!(expanded.contains("Opaque"), "{expanded}");
    assert!(expanded.contains("Candidates"), "{expanded}");
    assert!(expanded.contains("WhenRequired"), "{expanded}");
}
