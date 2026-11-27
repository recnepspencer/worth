//! Vocabulary enums keep their declared durable codes; malformed ones are
//! refused at the offending variant.
use super::vocabulary::expand;

fn expansion(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    let expanded = expand(&input).expect("the vocabulary is accepted");
    expanded.to_string().split_whitespace().collect()
}

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the vocabulary is refused")
        .to_string()
}

fn tokens(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn codes_are_the_declared_discriminants_and_the_binding_is_named_after_the_enum() {
    let declared = expansion(
        r#"#[worth_query(identity = "rebac.role.v1", bindings = Codecs)]
        pub enum Role { Reviewer = 1, Manager = 7 }"#,
    );
    assert!(declared.contains(&tokens("Role::Reviewer => 1u64")));
    assert!(declared.contains(&tokens("7u64 => ::core::result::Result::Ok(Self::Manager)")));
    assert!(declared.contains(&tokens("pub struct RoleBinding;")));
    assert!(declared.contains(&tokens(
        r#"const IDENTITY_NAME: &'static str = "rebac.role.v1";"#
    )));
    assert!(declared.contains(&tokens(
        "impl ::worth_query_decl::facade::application_schema::fragment::Scalar<Role> for Codecs"
    )));
    let renamed = expansion(
        r#"#[worth_query(identity = "rebac.role.v1", binding = Roles)] enum Role { Reviewer = 1 }"#,
    );
    assert!(renamed.contains(&tokens("struct Roles;")));
    assert!(!renamed.contains("Scalar<Role>"));
}

#[test]
fn missing_duplicate_and_non_unit_codes_are_refused() {
    assert!(
        refusal(r#"#[worth_query(identity = "x")] enum Role { Reviewer }"#)
            .contains("write the durable code of every vocabulary value")
    );
    assert!(
        refusal(r#"#[worth_query(identity = "x")] enum Role { A = 1, B = 1 }"#)
            .contains("vocabulary code 1 is used twice")
    );
    assert!(
        refusal(r#"#[worth_query(identity = "x")] enum Role { A(u8) = 1 }"#)
            .contains("unit variant")
    );
    assert!(refusal("enum Role { A = 1 }").contains("identity"));
    assert!(refusal(r#"#[worth_query(identity = "x", colour = 1)] enum Role { A = 1 }"#)
        .contains("unknown key for #[derive(WorthQueryVocabulary)]; expected one of: identity, binding, bindings"));
}
