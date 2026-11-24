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

#[test]
fn zero_string_selectors_are_emitted_on_row_and_aspect_while_keeping_aliases() {
    let input = syn::parse_str(
        "#[worth_query(bindings = C, aspect = TaskFacts, fragment = Board, entity = Task)] \
         pub struct TaskFactsValue { #[worth_query(column = \"TaskKey\")] pub key: u64, pub due_window: Option<u32> }",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(expanded.contains("pub type TaskKey ="), "{expanded}");
    assert!(expanded.contains("pub type DueWindow ="), "{expanded}");
    assert!(
        expanded.contains("pub const KEY : TaskKey = < TaskKey > :: at (\"TaskKey\") ;"),
        "{expanded}"
    );
    assert!(
        expanded.contains("pub const fn select_key () -> TaskKey"),
        "{expanded}"
    );
    assert!(
        expanded.contains("pub const fn select_due_window () -> DueWindow"),
        "{expanded}"
    );
    assert!(expanded.contains("impl TaskFacts"), "{expanded}");
    assert!(
        expanded.contains("pub const fn due_window () -> DueWindow"),
        "{expanded}"
    );
    assert!(
        expanded
            .contains("pub const DUE_WINDOW : DueWindow = < DueWindow > :: at (\"DueWindow\") ;"),
        "{expanded}"
    );
}

#[test]
fn uppercase_and_normalizing_record_fields_avoid_duplicate_selectors() {
    let upper = syn::parse_str(
        "#[worth_query(bindings = C, aspect = Metrics, fragment = Board, entity = Task)] \
         pub struct MetricRow { pub X: u64 }",
    )
    .unwrap();
    let expanded = expand(&upper).unwrap().to_string();
    assert!(
        expanded.contains(
            "impl MetricRow { # [allow (dead_code , non_upper_case_globals)] pub const X : X ="
        ),
        "{expanded}"
    );
    assert!(
        expanded.contains("pub const fn select_X () -> X"),
        "{expanded}"
    );
    let aspect_impl = expanded
        .split_once("impl Metrics")
        .map(|(_, rest)| rest)
        .expect("aspect impl exists");
    assert!(
        aspect_impl.contains("pub const fn X () -> X"),
        "{aspect_impl}"
    );
    assert!(!aspect_impl.contains("pub const X :"), "{aspect_impl}");

    let colliding = refusal(
        "#[worth_query(bindings = C, aspect = Metrics, fragment = Board, entity = Task)] \
         pub struct CollideRow { \
             #[worth_query(column = \"FooBar\")] pub foo_bar: u64, \
             #[worth_query(column = \"FooBarAlt\")] pub fooBar: u64 \
         }",
    );
    assert!(
        colliding.contains("duplicate selector constant `FOO_BAR`"),
        "{colliding}"
    );

    let explicit_collision = refusal(
        "#[worth_query(bindings = C, aspect = Metrics, fragment = Board, entity = Task)] \
         pub struct ExplicitCollideRow { \
             #[worth_query(selector_const = X)] pub X: u64 \
         }",
    );
    assert!(
        explicit_collision.contains("collides with aspect selector method `X()`"),
        "{explicit_collision}"
    );

    let disambiguated = syn::parse_str(
        "#[worth_query(bindings = C, aspect = Metrics, fragment = Board, entity = Task)] \
         pub struct CleanRow { \
             #[worth_query(column = \"FooBar\")] pub foo_bar: u64, \
             #[worth_query(column = \"FooBarAlt\", selector_const = FOO_BAR_ALT)] pub fooBar: u64 \
         }",
    )
    .unwrap();
    let expanded = expand(&disambiguated).unwrap().to_string();
    assert!(
        expanded.contains("pub const FOO_BAR : FooBar ="),
        "{expanded}"
    );
    assert!(
        expanded.contains("pub const FOO_BAR_ALT : FooBarAlt ="),
        "{expanded}"
    );
}
