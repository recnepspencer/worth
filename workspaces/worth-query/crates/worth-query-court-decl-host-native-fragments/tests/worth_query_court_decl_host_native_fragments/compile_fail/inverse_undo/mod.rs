#[test]
fn consumers_cannot_declare_or_invoke_inverse_undo() {
    let cases = trybuild::TestCases::new();
    let fixtures =
        "tests/worth_query_court_decl_host_native_fragments/compile_fail/inverse_undo/ui";
    cases.compile_fail(format!("{fixtures}/correction_derive.rs"));
    cases.compile_fail(format!("{fixtures}/application_home_undo.rs"));
}
