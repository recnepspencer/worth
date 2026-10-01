#[cfg(feature = "certification-test-authority")]
use worth_proof::AdmittedBlobReleaseProof;

#[test]
fn external_caller_cannot_mint_release_proof_from_observations() {
    trybuild::TestCases::new().compile_fail("tests/ui/blob_release_proof/private_fields.rs");
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn certification_issuer_binds_one_selected_publication() {
    let proof = AdmittedBlobReleaseProof::certification_admit(
        [1; 16], [2; 16], 3, [4; 16], 5, [6; 32], [7; 32],
    )
    .expect("complete selected identity and issuer evidence");
    assert_eq!(proof.store(), [1; 16]);
    assert_eq!(proof.object(), [2; 16]);
    assert_eq!(proof.generation(), 3);
    assert_eq!(proof.publication_allocation_epoch(), [4; 16]);
    assert_eq!(proof.publication_record_ordinal(), 5);
    assert_eq!(proof.publication_frame_sha256(), [6; 32]);
    assert_eq!(proof.issuer_evidence_sha256(), [7; 32]);
    assert!(AdmittedBlobReleaseProof::certification_admit(
        [1; 16], [2; 16], 3, [4; 16], 0, [6; 32], [7; 32],
    )
    .is_err());
}
