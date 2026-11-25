//! Malformed relations are refused and integrity laws lower to `Relation`.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the relation is refused")
        .to_string()
}

#[test]
fn malformed_relations_are_refused() {
    let cases = [
        (
            "#[worth_query(from = Project)] struct Contains;",
            "requires `#[worth_query(to = ..)]`",
        ),
        (
            "#[worth_query(to = Task, reference(Schema, Board))] struct Contains;",
            "also needs `from = Entity`",
        ),
        (
            "#[worth_query(to = Task, to = Other)] struct Contains;",
            "duplicate `to` key",
        ),
        (
            "#[worth_query(to = Task, acyclic = true)] struct Contains;",
            "is a flag and takes no value",
        ),
        (
            "#[worth_query(to = Task, relation)] struct Contains;",
            "unknown key for #[derive(WorthQueryRelation)]",
        ),
        (
            "#[worth_query(to = Task)] struct Contains { id: u64 }",
            "use a unit struct",
        ),
        (
            "#[worth_query(to = Task)] struct Contains<T>(T);",
            "does not support generic parameters",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

#[test]
fn relation_integrity_constants_and_reference_lower_cleanly() {
    let input = syn::parse_str(
        "#[worth_query(to = Task, from = Project, reference(Schema, Board), \
         acyclic, self_edges = false, symmetric, orbit, same_partition_endpoints, \
         unique = ApplicationRelationUniquenessScope::DirectedSemanticEdge, \
         cardinality = ApplicationRelationCardinality::at_most_one_target())] \
         pub struct Contains;",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    for needle in [
        "const NAME : & 'static str = \"Contains\"",
        "type To = Task",
        "const ACYCLIC : bool = true",
        "const SELF_EDGES : bool = false",
        "const SYMMETRIC : bool = true",
        "const ORBIT : bool = true",
        "const SAME_PARTITION_ENDPOINTS : bool = true",
        "const UNIQUE :",
        "const CARDINALITY :",
        "pub fn reference ()",
    ] {
        assert!(expanded.contains(needle), "{needle}: {expanded}");
    }
}
