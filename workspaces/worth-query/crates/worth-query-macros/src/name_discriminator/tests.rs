//! Discriminators expand into the encoder calls a hand-written impl makes;
//! malformed ones are refused with an error naming the fix.
use super::expand;

fn expansion(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    let expanded = expand(&input).expect("the discriminator is accepted");
    expanded.to_string().split_whitespace().collect()
}

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the discriminator is refused")
        .to_string()
}

/// `source` without whitespace, as `expansion` renders tokens.
fn tokens(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn an_enum_writes_its_variant_and_then_each_field() {
    let cut = expansion(
        "#[worth_query(identity = \"app.face.cut.v1\", referenced_roles = [\"Authored\"])] \
         enum Cut { Sided { cutter: NameRef<Face>, #[worth_query(name = \"side\")] which: Side }, \
         #[worth_query(name = \"cap\")] Cap(u32, bool), Whole }",
    );
    assert!(cut.contains(&tokens(
        r#"const DISCRIMINATOR_IDENTITY: &'static str = "app.face.cut.v1";"#
    )));
    assert!(cut.contains(&tokens(
        r#"fn referenced_roles() -> &'static [&'static str] { &["Authored"] }"#
    )));
    assert!(cut.contains(&tokens(
        r#"Self::Sided { cutter: field_0, which: field_1, } => {
            encoder.variant("Sided");
            encoder.field("cutter", field_0);
            encoder.field("side", field_1);
        }"#
    )));
    assert!(cut.contains(&tokens(
        r#"Self::Cap(field_0, field_1,) => {
            encoder.variant("cap");
            encoder.value(field_0);
            encoder.value(field_1);
        }"#
    )));
    assert!(cut.contains(&tokens(r#"Self::Whole => { encoder.variant("Whole"); }"#)));
}

#[test]
fn a_struct_writes_each_field_and_a_unit_struct_writes_none() {
    let named = expansion(
        "#[worth_query(identity = \"app.seam-of.v1\")] \
         struct SeamOf { index: u64, r#type: NameRef<Face> }",
    );
    assert!(named.contains(&tokens(
        r#"encoder.field("index", &self.index); encoder.field("type", &self.r#type);"#
    )));
    assert!(!named.contains("referenced_roles"));

    let tuple = expansion("#[worth_query(identity = \"app.pair.v1\")] struct Pair(u8, Side);");
    assert!(tuple.contains(&tokens("encoder.value(&self.0); encoder.value(&self.1);")));

    let unit = expansion("#[worth_query(identity = \"app.only.v1\")] struct Only;");
    assert!(unit.contains(&tokens(
        "encoder: &mut ::worth_query_decl::facade::application_naming::\
         ApplicationNameDiscriminatorEncoder,) { }"
    )));
}

#[test]
fn a_malformed_discriminator_is_refused() {
    let cases = [
        (
            "struct Side(u8);",
            "requires `#[worth_query(identity = ..)]`",
        ),
        (
            "#[worth_query(identity = \"\")] struct Side(u8);",
            "a discriminator identity is a dotted name",
        ),
        (
            "#[worth_query(identity = \"app..side\")] struct Side(u8);",
            "a discriminator identity is a dotted name",
        ),
        (
            "#[worth_query(identity = \".app.side\")] struct Side(u8);",
            "a discriminator identity is a dotted name",
        ),
        (
            "#[worth_query(identity = \"app side\")] struct Side(u8);",
            "a discriminator identity is a dotted name",
        ),
        (
            "#[worth_query(identity = app.side)] struct Side(u8);",
            "expected string literal",
        ),
        (
            "#[worth_query(identity = \"a\", identity = \"b\")] struct Side(u8);",
            "duplicate `identity` key",
        ),
        (
            "#[worth_query(identity = \"a\", referenced_roles = [\"r\", \"r\"])] struct Side(u8);",
            "duplicate referenced role `r`",
        ),
        (
            "#[worth_query(identity = \"a\", referenced_roles = \"r\")] struct Side(u8);",
            "expected square brackets",
        ),
        (
            "#[worth_query(identity = \"a\", roles = [\"r\"])] struct Side(u8);",
            "unknown key for #[derive(ApplicationNameDiscriminator)]",
        ),
        (
            "#[worth_query(identity = \"a\")] struct Side { #[worth_query(nested)] side: u8 }",
            "unknown key for #[derive(ApplicationNameDiscriminator)]",
        ),
        (
            "#[worth_query(identity = \"a\")] struct Side { \
             a: u8, #[worth_query(name = \"a\")] b: u8 }",
            "duplicate field name `a`",
        ),
        (
            "#[worth_query(identity = \"a\")] struct Side(#[worth_query(name = \"a\")] u8);",
            "a tuple field is written by position and has no name",
        ),
        (
            "#[worth_query(identity = \"a\")] enum Side { \
             Left, #[worth_query(name = \"Left\")] Right }",
            "duplicate variant name `Left`",
        ),
        (
            "#[worth_query(identity = \"a\")] enum Side { #[worth_query(name = \" \")] Left }",
            "a variant name is not empty and holds no whitespace",
        ),
        (
            "#[worth_query(identity = \"a\")] enum Side {}",
            "needs at least one variant",
        ),
        (
            "#[worth_query(identity = \"a\")] union Side { a: u8 }",
            "applies only to structs and enums",
        ),
        (
            "#[worth_query(identity = \"a\")] struct Side<T>(T);",
            "does not support generic parameters",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
