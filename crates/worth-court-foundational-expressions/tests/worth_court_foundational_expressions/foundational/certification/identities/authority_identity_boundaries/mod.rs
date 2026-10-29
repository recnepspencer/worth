#[allow(unused_imports)] use crate::certification::ordinary_context::identities::authority_identity_boundaries::*;
use {worth_foundational::admit_canonical_sequence_digest_derivation, worth_foundational::admit_foundational_authority_identity, worth_foundational::admit_foundational_external_identity_token, worth_foundational::derive_canonical_digest, worth_foundational::derive_foundational_digest_identity_evidence, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::project_foundational_identity, worth_foundational::readmit_foundational_authority_identity, worth_foundational::readmit_revalidated_foundational_authority_identity, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion, worth_foundational::FoundationalAdmittedIdentityValue, worth_foundational::FoundationalAuthorityIdentity, worth_foundational::FoundationalDigestIdentityEvidence, worth_foundational::FoundationalExternalIdentityToken, worth_foundational::FoundationalIdentityBasis, worth_foundational::FoundationalIdentityDigestDerivationEvidence, worth_foundational::FoundationalIdentityKind, worth_foundational::FoundationalIdentityProjectionEvidence, worth_foundational::FoundationalProjectionIdentity};
use {worth_proof::AuthorityMarker, worth_proof::AuthorityWitness, worth_proof::TransitionOutcome};

#[test]
fn authority_identity_requires_witness_and_preserves_value() {
    let admitted = admitted_commit_identity_value(42);
    let identity = crate::certification::ordinary_context::identities::authority_identity_boundaries::QueryCommitIdentity::from_admitted(admitted);

    assert_eq!(*identity.value(), 42);
}

#[test]
fn authority_identity_debug_redacts_raw_value() {
    let identity = commit_identity(42);
    let debug_output = format!("{identity:?}");

    assert!(debug_output.contains("FoundationalAuthorityIdentity"));
    assert!(debug_output.contains("authority-redacted"));
    assert!(!debug_output.contains("value: 42"));
}

#[test]
fn authority_identity_projects_without_losing_authority_type() {
    let identity = commit_identity(42);
    let projection_evidence: QueryCommitProjectionEvidence =
        worth_foundational::FoundationalIdentityProjectionEvidence::derive_from_authority(
            &identity,
            "query:commit:42".to_string(),
            authority(),
        );
    let projection: FoundationalProjectionIdentity<String, QueryCommitIdentityKind> =
        worth_foundational::FoundationalProjectionIdentity::from_projection_evidence(projection_evidence);

    assert_eq!(projection.label(), "query:commit:42");
    assert_eq!(identity.value(), &42);
}

#[test]
fn bridged_identity_requires_readmission_before_authority_use() {
    let identity = commit_identity(42);
    let bridged = identity.bridge_trust_boundary();
    let debug_output = format!("{bridged:?}");
    let revalidated = bridged.revalidate_current_value(authority());
    let readmitted = crate::certification::ordinary_context::identities::authority_identity_boundaries::QueryCommitIdentity::readmit(revalidated);

    assert!(debug_output.contains("FoundationalBoundaryBridgedIdentity"));
    assert!(debug_output.contains("boundary-bridged-redacted"));
    assert!(!debug_output.contains("value: 42"));
    assert_eq!(readmitted.value(), &42);
}

#[test]
fn bridged_identity_can_record_revalidated_value() {
    let identity = commit_identity(42);
    let bridged = identity.bridge_trust_boundary();
    let revalidated = bridged.revalidate_replacement_value(43, authority());
    let readmitted = crate::certification::ordinary_context::identities::authority_identity_boundaries::QueryCommitIdentity::readmit(revalidated);

    assert_eq!(readmitted.value(), &43);
}

#[test]
fn digest_evidence_stays_evidence_not_authority() {
    let identity = commit_identity(42);
    let derivation_evidence: QueryCommitDigestDerivationEvidence =
        worth_foundational::FoundationalIdentityDigestDerivationEvidence::derive_from_authority(
            &identity,
            identity_digest(),
            authority(),
        );
    let evidence: QueryCommitDigestEvidence =
        worth_foundational::FoundationalDigestIdentityEvidence::from_derivation_evidence(derivation_evidence);

    assert_eq!(evidence.digest().metadata().entry_count(), 1);
}

#[test]
fn external_token_needs_authority_admission() {
    let token = FoundationalExternalIdentityToken::<u64, QuerySnapshotIdentityKind>::new(77);
    let admitted = token.admit_with_authority::<QueryIdentityAuthority>(authority());
    let identity = worth_foundational::FoundationalAuthorityIdentity::from_admitted(admitted);

    assert_eq!(identity.value(), &77);
}

#[test]
fn lifecycle_helpers_preserve_explicit_authority_steps_with_less_boilerplate() {
    let identity: QueryCommitIdentity = admit_foundational_authority_identity(42, authority());
    let projection: FoundationalProjectionIdentity<String, QueryCommitIdentityKind> =
        project_foundational_identity(&identity, "query:commit:42".to_string(), authority());
    let digest_evidence: QueryCommitDigestEvidence =
        derive_foundational_digest_identity_evidence(&identity, identity_digest(), authority());
    let readmitted = readmit_foundational_authority_identity(
        identity.clone().bridge_trust_boundary(),
        authority(),
    );
    let revalidated = readmit_revalidated_foundational_authority_identity(
        identity.bridge_trust_boundary(),
        43,
        authority(),
    );

    assert_eq!(projection.label(), "query:commit:42");
    assert_eq!(digest_evidence.digest().metadata().entry_count(), 1);
    assert_eq!(readmitted.value(), &42);
    assert_eq!(revalidated.value(), &43);
}

#[test]
fn external_token_helper_admits_without_hiding_authority() {
    let token = FoundationalExternalIdentityToken::<u64, QuerySnapshotIdentityKind>::new(77);
    let identity = admit_foundational_external_identity_token::<
        u64,
        QueryIdentityAuthority,
        QuerySnapshotIdentityKind,
    >(token, authority());

    assert_eq!(identity.value(), &77);
}
