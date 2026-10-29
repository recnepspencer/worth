#[allow(unused_imports)]
pub(crate) use {};

#[test]
fn external_caller_cannot_mint_release_proof_from_observations() {
    trybuild::TestCases::new().compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/blob_release_proof/private_fields.rs"));
}
