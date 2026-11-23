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
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = Title, relation = Owns, \
                 nested = Unit)] owner: Child }}"
            ),
            "either a `field` or a `relation`, not both",
        ),
        (
            format!("{HEADER} struct Row {{ #[worth_query(relation = Owns)] owner: Child }}"),
            "add `nested = Binding`",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(relation = Owns, nested = Unit, \
                 column = \"Owner\")] owner: Child }}"
            ),
            "`column` does not apply to a relation slot",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = Title, reverse_from = Task)] \
                 title: String }}"
            ),
            "`reverse_from` does not apply to a field slot",
        ),
        (
            format!(
                "{HEADER} struct Row {{ #[worth_query(field = TaskKey, selector = key)] id: u64, \
                 #[worth_query(field = Title)] key: String }}"
            ),
            "selector `key` is already declared",
        ),
        (
            format!("{HEADER} struct Row {{ #[worth_query(field = TaskKey)] project_row: u64 }}"),
            "selector `project_row` is already declared",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

fn expansion(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input).expect("the projection expands").to_string()
}

/// The field type selects the traversal's cardinality and `reverse_from` its
/// direction; an overridden selector names every read of its slot.
#[test]
fn relation_slots_select_cardinality_from_the_field_type_and_direction_from_the_keys() {
    let expanded = expansion(&format!(
        "{HEADER} struct Row {{ \
         #[worth_query(relation = Contains, nested = Unit)] tasks: Vec<TaskRow>, \
         #[worth_query(relation = Owner, reverse_from = Person, nested = Unit)] owner: Option<(PersonRow)>, \
         #[worth_query(relation = Lead, nested = Unit, selector = lead_slot)] lead: PersonRow }}"
    ));
    for constructor in ["forward_many", "reverse_optional", "forward_one"] {
        assert!(expanded.contains(constructor), "{constructor}: {expanded}");
    }
    assert!(expanded.contains("fn lead_slot < Q >"), "{expanded}");
    assert!(
        expanded.contains("Self :: lead_slot :: < Q > ()"),
        "{expanded}"
    );
    assert!(!expanded.contains("fn lead < Q >"), "{expanded}");
    assert!(
        expanded.contains("< PersonRow > :: project_row"),
        "{expanded}"
    );
}
