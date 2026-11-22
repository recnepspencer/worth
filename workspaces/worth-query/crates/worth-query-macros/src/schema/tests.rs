//! Malformed schema markers are refused with an error naming the fix.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the marker is refused")
        .to_string()
}

#[test]
fn a_marker_without_exactly_one_role_or_with_a_foreign_key_is_refused() {
    let cases = [
        (
            "#[worth_query(name = \"board\")] struct Board;",
            "needs one role",
        ),
        (
            "#[worth_query(entity, fragment, aspects(A))] struct Task;",
            "exactly one role",
        ),
        (
            "#[worth_query(entity, owner = \"o\", aspects(A))] struct Task;",
            "`owner` does not apply to an `entity`",
        ),
        (
            "#[worth_query(fragment)] struct Board;",
            "requires `#[worth_query(members = ..)]`",
        ),
        (
            "#[worth_query(relation, to = B, reference(S, F))] struct Link;",
            "also needs `from = Entity`",
        ),
        (
            "#[worth_query(entity, aspects(A), aspects(B))] struct Task;",
            "duplicate `aspects` key",
        ),
        (
            "#[worth_query(entity, aspect(A))] struct Task;",
            "unknown key for #[derive(WorthQuerySchema)]",
        ),
        (
            "#[worth_query(entity = 1, aspects(A))] struct Task;",
            "is a flag and takes no value",
        ),
        (
            "#[worth_query(entity, aspects(A))] struct Task { key: u64 }",
            "use a unit struct",
        ),
        (
            "#[worth_query(entity, aspects(A))] struct Task<T>(T);",
            "does not support generic parameters",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
