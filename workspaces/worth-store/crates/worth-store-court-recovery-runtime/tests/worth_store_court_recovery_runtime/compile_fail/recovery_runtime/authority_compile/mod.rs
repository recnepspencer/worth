#[test]
fn recovery_entry_authority_is_compiler_sealed() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_store_court_recovery_runtime/sources/worth_store_recovery_runtime/ui/*.rs"
    ))
    .expect("prebuilt compiler denial");
}
