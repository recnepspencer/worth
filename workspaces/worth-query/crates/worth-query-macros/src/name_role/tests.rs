//! Role enums declare their canonical order; malformed ones are refused with
//! an error naming the fix.
use super::expand;

fn expansion(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    let expanded = expand(&input).expect("the role enum is accepted");
    expanded.to_string().split_whitespace().collect()
}

fn refusal(source: &str) -> String {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .expect_err("the role enum is refused")
        .to_string()
}

/// `source` without whitespace, as `expansion` renders tokens.
fn tokens(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn roles_keep_declaration_order_unless_a_dependency_points_forward() {
    let declared = expansion(
        "enum FaceRole { Authored, #[depends_on(Authored)] CutPiece, \
         #[depends_on(Authored, CutPiece)] Instance(u32) }",
    );
    assert!(declared.contains(&tokens(
        r#"const ROLE_ORDER: &'static [&'static str] = &["Authored", "CutPiece", "Instance"];"#
    )));
    assert!(declared.contains(&tokens(
        r#"&[("CutPiece", "Authored"), ("Instance", "Authored"), ("Instance", "CutPiece")];"#
    )));
    // An indexed role is born under its declared name alone, and carries its
    // index beside it.
    assert!(declared.contains(&tokens(r#"Self::Instance(_) => "Instance""#)));
    assert!(declared.contains(&tokens(
        "Self::Instance(index) => ::core::option::Option::Some(*index)"
    )));
    assert!(declared.contains(&tokens("Self::Instance(index) => (2u32, *index)")));
    assert!(declared.contains(&tokens("Self::CutPiece => (1u32, 0u32)")));

    // A role declared before what it depends on moves after it; the others
    // keep their declared order.
    let forward = expansion(
        "enum Role { #[depends_on(Seam)] Piece, Authored, #[depends_on(Authored)] Seam, Merged }",
    );
    assert!(forward.contains(&tokens(r#"&["Authored", "Seam", "Piece", "Merged"];"#)));
    assert!(forward.contains(&tokens("Self::Piece => (2u32, 0u32)")));
    assert!(forward.contains(&tokens("Self::Merged => (3u32, 0u32)")));
}

#[test]
fn an_explicit_name_keeps_a_persisted_role_name() {
    let renamed = expansion(
        "enum Role { #[worth_query(name = \"authored\")] Authored, \
         #[worth_query(name = \"instance\")] #[depends_on(Authored)] #[depends_on(r#Final)] \
         Instance(u32), r#Final }",
    );
    assert!(renamed.contains(&tokens(r#"&["authored", "Final", "instance"];"#)));
    assert!(renamed.contains(&tokens(
        r#"&[("instance", "authored"), ("instance", "Final")];"#
    )));
    assert!(renamed.contains(&tokens(r#"Self::Instance(_) => "instance""#)));
}

#[test]
fn a_malformed_role_enum_is_refused() {
    let cases = [
        (
            "enum Role { A, #[depends_on(Missing)] B }",
            "unknown role variant `Missing`; `Role` declares: A, B",
        ),
        (
            "enum Role { A, #[depends_on(A, A)] B }",
            "duplicate dependency `A`",
        ),
        (
            "enum Role { A, #[depends_on(A)] #[depends_on(A)] B }",
            "duplicate dependency `A`",
        ),
        (
            "enum Role { #[depends_on(A)] A }",
            "role `A` cannot depend on itself",
        ),
        (
            "enum Role { #[depends_on(B)] A, #[depends_on(A)] B }",
            "cyclic role dependencies: A -> B -> A",
        ),
        (
            "enum Role { Root, #[depends_on(C)] A, #[depends_on(A)] B, #[depends_on(B, Root)] C }",
            "cyclic role dependencies: A -> C -> B -> A",
        ),
        (
            "enum Role { A, #[depends_on()] B }",
            "names at least one role variant",
        ),
        (
            "enum Role { A, #[depends_on(\"A\")] B }",
            "expected identifier",
        ),
        ("enum Role { A, B(u64) }", "a role's index is a `u32`"),
        (
            "enum Role { A, B(u32, u32) }",
            "a unit, or carries one numeric index",
        ),
        (
            "enum Role { A, B { index: u32 } }",
            "a unit, or carries one numeric index",
        ),
        (
            "enum Role { #[worth_query(name = \"a\")] A, #[worth_query(name = \"a\")] B }",
            "duplicate role `a`",
        ),
        (
            "enum Role { #[worth_query(name = \"cut piece\")] A }",
            "a role name is not empty and holds no whitespace",
        ),
        (
            "enum Role { #[worth_query(name = \"\")] A }",
            "a role name is not empty and holds no whitespace",
        ),
        (
            "enum Role { #[worth_query(role = \"a\")] A }",
            "unknown key for #[derive(WorthQueryNameRole)]",
        ),
        (
            "#[worth_query(name = \"roles\")] enum Role { A }",
            "do not apply to the role enum; write them on a variant",
        ),
        (
            "#[depends_on(A)] enum Role { A }",
            "do not apply to the role enum; write them on a variant",
        ),
        (
            "enum Role { A(#[worth_query(name = \"index\")] u32) }",
            "do not apply to a role's index",
        ),
        ("enum Role {}", "needs at least one variant, each a role"),
        ("struct Role;", "applies only to enums"),
        (
            "enum Role<T> { A(T) }",
            "does not support generic parameters",
        ),
    ];
    for (source, expected) in cases {
        let message = refusal(source);
        assert!(message.contains(expected), "{source}: {message}");
    }
}
