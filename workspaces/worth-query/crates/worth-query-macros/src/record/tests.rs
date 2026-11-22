//! Malformed records are refused with an error naming the fix.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the record is refused")
        .to_string()
}

#[test]
fn a_record_missing_a_mapping_or_with_an_unsupported_shape_is_refused() {
    let seventeen: String = (0..17).map(|index| format!("f{index}: u64,")).collect();
    let too_wide = format!("#[worth_query(bindings = C, aspect = A)] struct Row {{ {seventeen} }}");
    let cases = [
        (
            "#[worth_query(aspect = A)] struct Row { key: u64 }".to_owned(),
            "requires `#[worth_query(bindings = ..)]`",
        ),
        (
            "#[worth_query(bindings = C)] struct Row { key: u64 }".to_owned(),
            "requires `#[worth_query(aspect = ..)]`",
        ),
        (
            "#[worth_query(bindings = C, aspect = A, fragment = F)] struct Row { key: u64 }"
                .to_owned(),
            "declare the column selections together",
        ),
        (
            "#[worth_query(bindings = C, aspect = A)] struct Row { \
             #[worth_query(column = \"Key\")] key: u64, #[worth_query(column = \"Key\")] other: u64 }"
                .to_owned(),
            "duplicate column `Key`",
        ),
        (
            "#[worth_query(bindings = C, aspect = A, fragment = F, entity = E)] struct Row { \
             #[worth_query(column = \"task-key\")] key: u64 }"
                .to_owned(),
            "column `task-key` is not a Rust type name",
        ),
        (
            "#[worth_query(bindings = C, aspect = A)] struct Row { #[worth_query(equal)] key: u64 }"
                .to_owned(),
            "unknown key for #[derive(WorthQueryRecord)]",
        ),
        (
            "#[worth_query(bindings = C, aspect = A)] struct Row {}".to_owned(),
            "at least one field",
        ),
        (too_wide, "at most 16 fields"),
        (
            "#[worth_query(bindings = C, aspect = A)] struct Row(u64);".to_owned(),
            "requires a struct with named fields",
        ),
        (
            "#[worth_query(bindings = C, aspect = A)] enum Row { Key }".to_owned(),
            "applies only to structs",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
