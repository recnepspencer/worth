#[test]
fn external_caller_cannot_mint_release_proof_from_observations() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_proof/sources/worth_proof/ui/blob_release_proof/private_fields.rs"
    ))
    .expect("prebuilt compiler denial");
}
