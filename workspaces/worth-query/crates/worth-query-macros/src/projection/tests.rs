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

/// A name slot reads the row's own name and takes no slot key.
#[test]
fn a_name_slot_projects_the_root_name_and_refuses_slot_keys() {
    let expanded = expansion(&format!(
        "{HEADER} struct Row {{ \
         #[worth_query(name)] name: WorthQueryName<Task>, \
         #[worth_query(name)] alias: Option<WorthQueryName<Task>>, \
         #[worth_query(field = Title, name = \"heading\")] title: String, \
         #[worth_query(relation = Contains, nested = Unit)] tasks: Vec<TaskRow> }}"
    ));
    assert!(
        expanded.contains("name : row . name :: < Task > () ?"),
        "{expanded}"
    );
    assert!(
        expanded.contains("alias : row . optional_name :: < Task > () ?"),
        "{expanded}"
    );
    assert!(
        expanded.contains(
            "const PROJECTS_PERSISTENT_NAMES : bool = true || < TaskRow > :: PROJECTS_PERSISTENT_NAMES ;"
        ),
        "{expanded}"
    );
    assert!(expanded.contains("\"heading\""), "{expanded}");
    let grouped = expansion(&format!(
        "{HEADER} struct Row {{ \
         #[worth_query(group_name)] shell: WorthQueryName<Shell>, \
         #[worth_query(group_name)] zone: Option<WorthQueryName<Zone>> }}"
    ));
    assert!(
        grouped.contains("shell : row . group_name () ?"),
        "{grouped}"
    );
    assert!(
        grouped.contains("zone : row . optional_group_name () ?"),
        "{grouped}"
    );
    assert!(
        grouped.contains("const PROJECTS_PERSISTENT_NAMES : bool = true ;"),
        "{grouped}"
    );
    // A name is not a result slot: no marker, selector or shape member.
    assert!(!expanded.contains("struct Name ;"), "{expanded}");
    assert!(!expanded.contains("fn name < Q >"), "{expanded}");

    let plain = expansion(&format!(
        "{HEADER} struct Row {{ #[worth_query(field = Title)] title: String }}"
    ));
    assert!(
        plain.contains("const PROJECTS_PERSISTENT_NAMES : bool = false ;"),
        "{plain}"
    );

    for (keys, expected) in [
        (
            "name, field = Title",
            "`field` does not apply to a name slot",
        ),
        (
            "name, slot = \"a.b\"",
            "`slot` does not apply to a name slot",
        ),
        ("name, name = \"n\"", "`name` does not apply to a name slot"),
        ("name, name", "duplicate `name` key"),
        (
            "group_name, field = Title",
            "`field` does not apply to a name slot",
        ),
        (
            "name, group_name",
            "either the row's `name` or its `group_name`",
        ),
        ("group_name, group_name", "duplicate `group_name` key"),
        ("grouped", "unknown key for #[derive(WorthQueryProjection)]"),
    ] {
        let message = refusal(&format!(
            "{HEADER} struct Row {{ #[worth_query({keys})] name: WorthQueryName<Task> }}"
        ));
        assert!(message.contains(expected), "{keys}: {message}");
    }
}
