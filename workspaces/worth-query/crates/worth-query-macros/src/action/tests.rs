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
            format!("#[worth_query(scope_value = rules::workspace)] {HEADER} struct Rename {{ task: u64 }}"),
            "requires `#[worth_query(scope = ..)]`",
        ),
        (
            format!("#[worth_query(scope = TaskKey)] {HEADER} struct Rename {{ task: u64 }}"),
            "requires `#[worth_query(scope_value = ..)]`",
        ),
        (
            format!(
                "#[worth_query(scope = TaskKey, scope_value = rules::workspace, principal_scope = Owner)] {HEADER} \
                 struct Rename {{ task: u64 }}"
            ),
            "exactly one scope",
        ),
        (
            format!(
                "#[worth_query(scope = TaskKey, scope_value = rules::workspace)] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "exactly one scope",
        ),
        (
            format!(
                "#[worth_query(scope = TaskKey, scope_const = 7, scope_value = rules::inbox)] \
                 {HEADER} struct Rename {{ task: u64 }}"
            ),
            "not both",
        ),
        (
            format!(
                "#[worth_query(scope_const = 7, principal_scope = Owner)] {HEADER} \
                 struct Rename {{ task: u64 }}"
            ),
            "exactly one scope",
        ),
        (
            format!(
                "#[worth_query(scope_column = \"Key\")] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "`scope_column` requires struct-level",
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
        (
            format!(
                "#[worth_query(validate = rules::title, validate = rules::other)] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "duplicate `validate` key",
        ),
        (
            format!(
                "#[worth_query(validate = \"rules::title\")] {HEADER} \
                 struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
            ),
            "`validate` takes a function path",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(&source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

/// The validator reaches the input binding as the hand-written `validate:`
/// clause, after `capture_input` when the handler is an authored body.
#[test]
fn a_validator_is_forwarded_to_the_input_binding_of_body_and_handler_actions() {
    for (flags, clause) in [
        ("validate = rules::title", "identity : "),
        ("body, validate = rules::title", "capture_input : true"),
    ] {
        let source = format!(
            "#[worth_query({flags})] {HEADER} \
             struct Rename {{ #[worth_query(scope = TaskKey)] task: u64 }}"
        );
        let input = syn::parse_str(&source).expect("the court input parses");
        let expansion = expand(&input).expect("the action expands").to_string();
        // The binding's body is the first brace group after its invocation.
        let binding = expansion
            .split_once("worth_query_structured_value_binding !")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(body, _)| body)
            .expect("the input binding is declared");
        assert!(binding.contains(clause), "{binding}");
        assert!(binding.contains(", validate : rules :: title"), "{binding}");
        assert!(
            binding.contains(", fields : [required (\"task\" , task : u64)]"),
            "{binding}"
        );
    }
}

#[test]
fn a_scope_value_function_and_body_defaults_and_typed_inputs_expand_cleanly() {
    let fixed_scope = syn::parse_str(&format!(
        "#[worth_query(scope = TaskKey, scope_value = rules::inbox)] {HEADER} \
         struct NewProject;"
    ))
    .unwrap();
    let expanded = expand(&fixed_scope).unwrap().to_string();
    assert!(expanded
        .contains("scope TaskKey , field < TaskKey > :: at (\"TaskKey\") , value rules :: inbox"));
    assert!(!expanded.contains("worth_query_scope"));

    let constant_scope = syn::parse_str(&format!(
        "#[worth_query(scope = DutyKey, scope_const = duty::RECEIVING)] {HEADER} \
         struct OpenDock;"
    ))
    .unwrap();
    let expanded = expand(&constant_scope).unwrap().to_string();
    assert!(
        expanded.contains("value OpenDock :: worth_query_scope"),
        "{expanded}"
    );
    assert!(expanded.contains("{ duty :: RECEIVING }"), "{expanded}");

    let body_default = syn::parse_str(
        "#[worth_query(schema = Schema, context = Ctx, namespace = NS, \
         input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct Reschedule { #[worth_query(scope = TaskKey)] task: u64, due: Option<CanonicalDate>, title: String }",
    )
    .unwrap();
    let expanded = expand(&body_default).unwrap().to_string();
    assert!(
        expanded.contains("UnitApplicationValueBinding"),
        "{expanded}"
    );
    assert!(
        expanded.contains("WorthQueryOperationBodyDecision"),
        "{expanded}"
    );
    assert!(
        expanded.contains("read_fields : [< TaskKey > :: at (\"TaskKey\")]"),
        "{expanded}"
    );
    assert!(
        expanded.contains("fn due_input () -> :: worth_query_decl :: facade :: application_schema :: ApplicationOptionalInputField < Self , CanonicalDate >"),
        "{expanded}"
    );
    assert!(
        expanded.contains("fn title_input () -> :: worth_query_decl :: facade :: application_schema :: ApplicationInputField < Self , String >"),
        "{expanded}"
    );
    assert!(
        expanded.contains("required (\"title\" , title : String)"),
        "{expanded}"
    );
    assert!(
        expanded.contains("optional (\"due\" , due : CanonicalDate)"),
        "{expanded}"
    );
    assert!(
        expanded.contains("ApplicationInputField :: for_binding :: < In > (\"title\")"),
        "{expanded}"
    );
    assert!(
        expanded.contains("# [allow (clippy :: clone_on_copy)]"),
        "{expanded}"
    );

    let renamed = syn::parse_str(
        "#[serde(rename_all = \"camelCase\")] \
         #[worth_query(schema = Schema, context = Ctx, namespace = NS, \
          input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct RenameWire { \
             #[worth_query(scope = TaskKey)] task: u64, \
             #[serde(alias = \"legacy\", rename = \"stored_title\")] new_title: String, \
             #[serde(default = \"default_note\", rename = \"stored_note\")] note: String, \
             #[serde(skip_serializing_if = \"Option::is_none\", rename = \"stored_opt\")] opt: Option<String>, \
             due_window: Option<u32>, \
             #[serde(skip)] transient: u8 \
         }",
    )
    .unwrap();
    let expanded = expand(&renamed).unwrap().to_string();
    for needle in [
        "(\"stored_title\"",
        "(\"stored_note\"",
        "(\"stored_opt\"",
        "(\"dueWindow\"",
    ] {
        assert!(expanded.contains(needle), "{needle}: {expanded}");
    }
    assert!(!expanded.contains("transient_input"), "{expanded}");

    // Serde `apply_to_field` treats both `lowercase` and `snake_case` as identity on struct
    // fields (preserving `fooBar` distinct from `foobar` and `foo_bar`), and uppercases without
    // splitting camelCase under `SCREAMING_SNAKE_CASE`.
    let lower_rule = syn::parse_str(
        "#[serde(rename_all = \"lowercase\")] \
         #[worth_query(schema = Schema, context = Ctx, namespace = NS, \
          input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct LowerWire { \
             #[worth_query(scope = TaskKey)] task: u64, \
             fooBar: String, \
             foobar: String, \
         }",
    )
    .unwrap();
    let expanded = expand(&lower_rule).unwrap().to_string();
    assert!(expanded.contains("(\"fooBar\""), "{expanded}");
    assert!(expanded.contains("(\"foobar\""), "{expanded}");

    let snake_rule = syn::parse_str(
        "#[serde(rename_all = \"snake_case\")] \
         #[worth_query(schema = Schema, context = Ctx, namespace = NS, \
          input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct SnakeWire { \
             #[worth_query(scope = TaskKey)] task: u64, \
             fooBar: String, \
             foo_bar: String, \
         }",
    )
    .unwrap();
    let expanded = expand(&snake_rule).unwrap().to_string();
    assert!(expanded.contains("(\"fooBar\""), "{expanded}");
    assert!(expanded.contains("(\"foo_bar\""), "{expanded}");

    let screaming_rule = syn::parse_str(
        "#[serde(rename_all = \"SCREAMING_SNAKE_CASE\")] \
         #[worth_query(schema = Schema, context = Ctx, namespace = NS, \
          input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct ScreamingWire { \
             #[worth_query(scope = TaskKey)] task: u64, \
             fooBar: String, \
             foo_bar: String, \
         }",
    )
    .unwrap();
    let expanded = expand(&screaming_rule).unwrap().to_string();
    assert!(expanded.contains("(\"FOOBAR\""), "{expanded}");
    assert!(expanded.contains("(\"FOO_BAR\""), "{expanded}");

    let dup_wire = refusal(
        "#[worth_query(schema = Schema, context = Ctx, namespace = NS, \
          input = In, operation = Op, binding = Bind, body, reads(Task))] \
         struct DupWire { \
             #[worth_query(scope = TaskKey)] task: u64, \
             #[serde(rename = \"same\")] a: String, \
             #[serde(rename = \"same\")] b: String, \
         }",
    );
    assert!(
        dup_wire.contains("duplicate serialized input member name"),
        "{dup_wire}"
    );

    for bad_attr in [
        "#[serde(transparent)]",
        "#[serde(into = \"Wire\")]",
        "#[serde(from = \"Wire\")]",
        "#[serde(try_from = \"Wire\")]",
    ] {
        let err = refusal(&format!(
            "{bad_attr} \
             #[worth_query(schema = Schema, context = Ctx, namespace = NS, \
              input = In, operation = Op, binding = Bind, body, reads(Task))] \
             struct Bad {{ #[worth_query(scope = TaskKey)] task: u64 }}"
        ));
        assert!(err.contains("container rewrites"), "{bad_attr}: {err}");
    }
}
