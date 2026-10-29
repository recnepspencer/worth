#[allow(unused_imports)]
pub(crate) use crate::fast::blob_release_proof_contract::*;

#[test]
fn external_caller_cannot_mint_release_proof_from_observations() {
    trybuild::TestCases::new().compile_fail("tests/ui/blob_release_proof/private_fields.rs");
}
