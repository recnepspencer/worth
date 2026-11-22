//! Malformed actions are refused with an error naming the fix.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the action is refused")
        .to_string()
}

const HEADER: &str = "#[worth_query(schema = S, context = C, namespace = NS, input = I, \
     operation = O, binding = B, result = R, decision = D)]";

#[test]
fn an_action_without_exactly_one_scope_or_with_a_malformed_key_is_refused() {
    let cases = [
        (
            format!("{HEADER} struct Rename {{ task: u64 }}"),
            "missing action scope",
        ),
        (
            format!(
                "{HEADER} struct Rename {{ #[worth_query(scope = TaskKey)] task: u64, \
                 #[worth_query(scope = Title)] title: String }}"
            ),
            "exactly one scope",
        ),
        (
            "#[worth_query(schema = S, context = C, namespace = NS, input = I, operation = O, \
             binding = B, result = R, decision = D, principal_scope = Owner)] \
             struct Rename { #[worth_query(scope = TaskKey)] task: u64 }"
                .to_owned(),
            "exactly one scope",
        ),
        (
            format!("{HEADER} struct Rename {{ #[worth_query(column = \"Key\")] task: u64 }}"),
            "names the column of a `scope` field",
        ),
        (
            "#[worth_query(schema = S, context = C, input = I, operation = O, binding = B, \
             result = R, decision = D)] struct Rename { #[worth_query(scope = TaskKey)] task: u64 }"
                .to_owned(),
            "requires `#[worth_query(namespace = ..)]`",
        ),
        (
            format!(
                "#[worth_query(reads(Task), reads(Task))] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "duplicate `reads` key",
        ),
        (
            format!(
                "#[worth_query(body = true)] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "is a flag and takes no value",
        ),
        (
            format!(
                "#[worth_query(handler = H)] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "unknown key for #[derive(WorthQueryAction)]",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
