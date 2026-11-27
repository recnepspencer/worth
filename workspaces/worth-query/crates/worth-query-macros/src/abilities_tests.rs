//! Ability enums declare one marker per variant through `worth_query_ability!`.
use super::abilities::expand;

fn expansion(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    let expanded = expand(&input).expect("the abilities are accepted");
    expanded.to_string().split_whitespace().collect()
}

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the abilities are refused")
        .to_string()
}

fn tokens(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn each_variant_is_a_marker_in_the_snake_case_module() {
    let declared = expansion(
        "#[worth_query(schema = Board, scoped_to = Case)] pub enum CaseAbility { Annotate, Close }",
    );
    assert!(declared.contains(&tokens("pub mod case_ability")));
    assert!(declared.contains(&tokens(
        "::worth_query_decl::facade::worth_query_ability!(pub Annotate scoped_to Case, in Board);"
    )));
    assert!(declared.contains(&tokens(
        "const ALL: &'static [Self] = &[Self::Annotate, Self::Close];"
    )));
    assert!(declared.contains(&tokens(
        "Self::Close => case_ability::Close::reference().name(),"
    )));
    let named = expansion(
        "#[worth_query(schema = Board, scoped_to = Case, module = markers)] enum Ability { Read }",
    );
    assert!(named.contains(&tokens("mod markers")));
}

#[test]
fn coded_or_carrying_variants_and_missing_keys_are_refused() {
    assert!(
        refusal("#[worth_query(schema = Board, scoped_to = Case)] enum A { Read = 1 }")
            .contains("its name is its identifier")
    );
    assert!(
        refusal("#[worth_query(schema = Board, scoped_to = Case)] enum A { Read(u8) }")
            .contains("unit variant")
    );
    assert!(refusal("#[worth_query(schema = Board)] enum A { Read }").contains("scoped_to"));
}
