//! Malformed projections are refused with an error naming the fix.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the projection is refused")
        .to_string()
}

const HEADER: &str = "#[worth_query(schema = S, root = Task, namespace = READ)]";

#[test]
fn a_projection_missing_a_mapping_or_repeating_a_slot_is_refused() {
    let cases = [
        (
            format!("{HEADER} struct Row {{ key: u64 }}"),
            "missing `#[worth_query(field = Selection)]`",
        ),
        (
            "#[worth_query(schema = S, root = Task)] struct Row { \
             #[worth_query(field = TaskKey)] key: u64 }"
                .to_owned(),
            "slot `key` has no identity",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = TaskKey)] key: u64, \
                 #[worth_query(field = Title, name = \"key\")] title: String }}"
            ),
            "duplicate slot `key`",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = TaskKey, slot = \"a.b\")] key: u64, \
                 #[worth_query(field = Title, slot = \"a.b\")] title: String }}"
            ),
            "duplicate slot identity `a.b`",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = TaskKey, name = \"due-on\")] key: u64 }}"
            ),
            "cannot name its slot marker",
        ),
        (
            format!("{HEADER} struct Row {{ #[worth_query(field = (u64, u64))] key: u64 }}"),
            "add `column = \"..\"`",
        ),
        (
            "#[worth_query(root = Task, namespace = READ)] struct Row { \
             #[worth_query(field = TaskKey)] key: u64 }"
                .to_owned(),
            "requires `#[worth_query(schema = ..)]`",
        ),
        (
            format!("{HEADER} struct Row {{ #[worth_query(selection = TaskKey)] key: u64 }}"),
            "unknown key for #[derive(WorthQueryProjection)]",
        ),
        (
            format!("{HEADER} struct Row<'a> {{ #[worth_query(field = Title)] title: &'a str }}"),
            "does not support generic parameters",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
