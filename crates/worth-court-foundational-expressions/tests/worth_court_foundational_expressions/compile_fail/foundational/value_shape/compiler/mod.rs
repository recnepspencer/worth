// Compile the supported counterpart and denied grammars in one fixture session.

#[test]
fn derive_accepts_bounded_values_and_rejects_unsupported_grammars() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_foundational_expressions/sources/worth_foundational/value_shape/ui/fail/*.rs")).expect("prebuilt compiler denial");
}
