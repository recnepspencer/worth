//! Grammar and hygiene unit tests for `#[derive(WorthBinaryValue)]`.
use super::expand;

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("test input parses");
    expand(&input)
        .expect_err("input must be refused")
        .to_string()
}

#[test]
fn malformed_binary_values_are_refused() {
    let cases = [
        (
            "struct MissingIdentity { x: i64 }",
            "requires `#[worth_binary(namespace = \"..\", version = 1)]`",
        ),
        (
            "#[worth_binary(identity = \"a.v1\", namespace = \"a\")] struct BothIdNs { x: i64 }",
            "specify either `identity = \"..\"` or `namespace = \"..\"`, not both",
        ),
        (
            "#[worth_binary(namespace = \"a\")] struct BadFieldAttr { #[worth_binary(version = 1)] x: i64 }",
            "does not accept field-level",
        ),
        (
            "#[worth_query(identity = \"\")] struct EmptyId { x: i64 }",
            "`identity` must be non-empty",
        ),
        (
            "#[worth_query(identity = \"a b\")] struct SpaceId { x: i64 }",
            "`identity` must be non-empty and contain no whitespace",
        ),
        (
            "#[worth_query(identity = \"a.v1\", version = 0)] struct ZeroVer { x: i64 }",
            "`version` must be at least 1",
        ),
        (
            "#[worth_query(identity = \"a.v1\", version = 256)] struct BigVer { x: i64 }",
            "`version` must be an integer in 1..=255",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] struct BadF32 { x: f32 }",
            "refuses raw `f32`",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] struct BadF64 { x: f64 }",
            "refuses raw `f64`",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] enum NotStruct { A }",
            "enums are not supported",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] union NotStruct { a: u32 }",
            "unions are not supported",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] struct Generic<T> { a: T }",
            "does not support generic parameters",
        ),
        (
            "#[worth_query(identity = \"a.v1\")] struct BadAttr { #[worth_query(skip)] a: i64 }",
            "does not accept field-level",
        ),
        (
            "#[worth_query(identity = \"a.v1\", nope = 1)] struct BadKey { a: i64 }",
            "unknown key for #[derive(WorthBinaryValue)]",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}

#[test]
fn valid_binary_value_expands_with_version_hygiene_and_default_binding() {
    let input = syn::parse_str(
        "#[worth_query(identity = \"shopfloor.query_box.v1\", version = 2, validate = check_box)] \
         pub struct QueryBox2D { pub min_x: i64, pub min_y: i64, pub max_x: i64, pub max_y: i64 }",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(
        expanded.contains("pub struct QueryBox2DBinding"),
        "{expanded}"
    );
    assert!(expanded.contains("BinaryValueDelegate"), "{expanded}");
    assert!(expanded.contains("WorthBinaryWireField"), "{expanded}");
    assert!(
        expanded.contains("WorthQueryDefaultValueBinding"),
        "{expanded}"
    );
    assert!(expanded.contains("UnsupportedVersion"), "{expanded}");
    assert!(
        expanded.contains("worth_binary_checked_wire_bytes"),
        "{expanded}"
    );
    assert!(expanded.contains("check_box"), "{expanded}");

    let target_dx = syn::parse_str(
        "#[worth_binary(namespace = \"daymark.focus_window\", version = 3)] \
         pub struct FocusWindow { pub start: i64, pub end: i64 }",
    )
    .unwrap();
    let expanded_ns = expand(&target_dx).unwrap().to_string();
    assert!(
        expanded_ns.contains("\"daymark.focus_window.v3\""),
        "{expanded_ns}"
    );
}

#[test]
fn raw_identifier_and_tuple_struct_expand_cleanly() {
    let input = syn::parse_str(
        "#[worth_query(identity = \"shopfloor.tag.v1\", binding = TagCodecBinding)] \
         pub struct Tag(pub [u8; 16], pub bool);",
    )
    .unwrap();
    let expanded = expand(&input).unwrap().to_string();
    assert!(
        expanded.contains("pub struct TagCodecBinding"),
        "{expanded}"
    );
    assert!(
        expanded.contains("0 :") && expanded.contains("1 :") && expanded.contains(". 0"),
        "{expanded}"
    );
    let unit = syn::parse_str(
        "#[worth_binary(namespace = \"shopfloor.unit\", version = 1)] pub struct UnitTag;",
    )
    .unwrap();
    let unit_expanded = expand(&unit).unwrap().to_string();
    assert!(
        unit_expanded.contains("__WorthBinaryValue { }"),
        "{unit_expanded}"
    );
}
