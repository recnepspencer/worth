//! A real enum pair admits vocabulary while genuine/ambiguous roots stay refused.
use super::*;
fn fixture(code: &str, extra: &str) -> Inputs {
    let mut input=Inputs::fixture(&[
        ("crates/a/Cargo.toml","[package]\nname='a'"),
        ("crates/a/src/lib.rs","#[cfg(test)] mod tests;"),
        ("crates/a/src/tests.rs",code),
        ("crates/a/src/denial.rs", "pub enum WorthQueryApplicationRequestQueryDenial { #[example] ExecutionRequest(Stop), Other { execution: () } } enum Runtime { ExecutionRequest }"),
        ("crates/a/src/shadow.rs",extra),
    ],"");
    input.files.insert(
        format!("{CONFIG}runtime-roots.toml"),
        "[[roots]]\ntype='ExecutionRequest'\n[[roots]]\ntype='Runtime'".into(),
    );
    input
        .files
        .insert(format!("{CONFIG}src-court-baseline.json"), "[]".into());
    input
}
#[test]
fn actual_qualified_variant_is_value_but_payloads_and_unknown_roots_are_not() {
    let good="fn test() { matches!(error, WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_)); }";
    assert!(check(&fixture(good, "")).is_empty());
    assert!(check(&fixture(
        good,
        "type Source = Result<(), WorthQueryApplicationRequestQueryDenial>;"
    ))
    .is_empty());
    assert!(check(&fixture(
        good,
        "type Handler = fn(<WorthQueryApplicationRequestQueryDenial as Trait>::Item);"
    ))
    .is_empty());
    for bad in [
        "fn test() { ExecutionRequest::serial(&source); }",
        "fn test() { let _: ExecutionRequest; }",
        "use worth_execution::ExecutionRequest;",
        "fn test() { Unknown::ExecutionRequest(()); }",
        "fn test() { Runtime::ExecutionRequest(()); }",
        "fn test() { WorthQueryApplicationRequestQueryDenial::ExecutionRequest(ExecutionRequest::serial(&source)); }",
        "fn test() { WorthQueryApplicationRequestQueryDenial::ExecutionRequest::serial(&source); }",
    ] {
        assert!(!check(&fixture(bad,"")).is_empty(),"{bad}");
    }
}
#[test]
fn ambiguous_enum_owner_or_shadowing_alias_keeps_conservative_runtime_finding() {
    let code = "fn test() { WorthQueryApplicationRequestQueryDenial::ExecutionRequest(()); }";
    for extra in [
        "struct WorthQueryApplicationRequestQueryDenial;",
        "type WorthQueryApplicationRequestQueryDenial = Other;",
        "use other::Value as WorthQueryApplicationRequestQueryDenial;",
        "enum WorthQueryApplicationRequestQueryDenial { ExecutionRequest }",
        "fn source<WorthQueryApplicationRequestQueryDenial>() {}",
        "struct Source<WorthQueryApplicationRequestQueryDenial>(());",
        "impl<WorthQueryApplicationRequestQueryDenial> Source {}",
    ] {
        assert!(!check(&fixture(code, extra)).is_empty(), "{extra}");
    }
}

#[test]
fn payload_types_and_named_fields_cannot_forge_a_declared_variant() {
    let code = "fn test() { PayloadOnly::ExecutionRequest(()); }";
    for extra in [
        "enum PayloadOnly { Value(ExecutionRequest) }",
        "enum PayloadOnly { Named { ExecutionRequest: () } }",
    ] {
        assert!(!check(&fixture(code, extra)).is_empty(), "{extra}");
    }
}
