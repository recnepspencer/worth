//! Compile the supported counterpart and denied grammars in one fixture session.

#[test]
fn derive_accepts_bounded_values_and_rejects_unsupported_grammars() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/value_shape/ui/pass/nested_value.rs");
    tests.compile_fail("tests/value_shape/ui/fail/*.rs");
}
