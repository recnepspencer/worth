use worth_foundational::{
    admit_canonical_sequence_digest_derivation, admit_foundational_authority_identity,
    admit_foundational_external_identity_token, derive_canonical_digest,
    derive_foundational_digest_identity_evidence, prepare_canonical_basis_sequence,
    project_foundational_identity, readmit_foundational_authority_identity,
    readmit_revalidated_foundational_authority_identity, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalIntegerWidth, CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion,
    FoundationalAdmittedIdentityValue, FoundationalAuthorityIdentity,
    FoundationalDigestIdentityEvidence, FoundationalExternalIdentityToken,
    FoundationalIdentityBasis, FoundationalIdentityDigestDerivationEvidence,
    FoundationalIdentityKind, FoundationalIdentityProjectionEvidence,
    FoundationalProjectionIdentity,
};
use worth_proof::{AuthorityMarker, AuthorityWitness, TransitionOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

struct QuerySnapshotIdentityKind;
impl FoundationalIdentityKind for QuerySnapshotIdentityKind {}

struct QueryCommitIdentityBasis;
impl FoundationalIdentityBasis for QueryCommitIdentityBasis {}

type QueryCommitIdentity =
    FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>;

type QueryCommitAdmittedValue =
    FoundationalAdmittedIdentityValue<u64, QueryIdentityAuthority, QueryCommitIdentityKind>;

type QueryCommitProjectionEvidence =
    FoundationalIdentityProjectionEvidence<String, QueryIdentityAuthority, QueryCommitIdentityKind>;

type QueryCommitDigestDerivationEvidence = FoundationalIdentityDigestDerivationEvidence<
    QueryCommitIdentityBasis,
    QueryIdentityAuthority,
    QueryCommitIdentityKind,
>;

type QueryCommitDigestEvidence = FoundationalDigestIdentityEvidence<
    QueryCommitIdentityBasis,
    QueryIdentityAuthority,
    QueryCommitIdentityKind,
>;

fn authority() -> AuthorityWitness<QueryIdentityAuthority> {
    AuthorityWitness::from_authority_marker(QueryIdentityAuthority(()))
}

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

fn identity_digest() -> worth_foundational::CanonicalDerivedDigest {
    let version = version("authority.identity.boundary");
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Identity,
        CanonicalBasisLocus::Named("query.commit".into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 42,
        },
    );
    let sequence = match prepare_canonical_basis_sequence(
        version.clone(),
        CanonicalBasisDomain::Identity,
        [entry],
    ) {
        TransitionOutcome::Success(sequence) => sequence,
        _ => panic!("identity basis should be ready"),
    };
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Identity,
        version,
    );
    match admit_canonical_sequence_digest_derivation(sequence, slot) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("identity digest derivation should be admitted"),
    }
}

fn admitted_commit_identity_value(value: u64) -> QueryCommitAdmittedValue {
    FoundationalAdmittedIdentityValue::admit(value, authority())
}

fn commit_identity(value: u64) -> QueryCommitIdentity {
    QueryCommitIdentity::from_admitted(admitted_commit_identity_value(value))
}

#[test]
fn authority_identity_requires_witness_and_preserves_value() {
    let admitted = admitted_commit_identity_value(42);
    let identity = QueryCommitIdentity::from_admitted(admitted);

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
        FoundationalIdentityProjectionEvidence::derive_from_authority(
            &identity,
            "query:commit:42".to_string(),
            authority(),
        );
    let projection: FoundationalProjectionIdentity<String, QueryCommitIdentityKind> =
        FoundationalProjectionIdentity::from_projection_evidence(projection_evidence);

    assert_eq!(projection.label(), "query:commit:42");
    assert_eq!(identity.value(), &42);
}

#[test]
fn bridged_identity_requires_readmission_before_authority_use() {
    let identity = commit_identity(42);
    let bridged = identity.bridge_trust_boundary();
    let debug_output = format!("{bridged:?}");
    let revalidated = bridged.revalidate_current_value(authority());
    let readmitted = QueryCommitIdentity::readmit(revalidated);

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
    let readmitted = QueryCommitIdentity::readmit(revalidated);

    assert_eq!(readmitted.value(), &43);
}

#[test]
fn digest_evidence_stays_evidence_not_authority() {
    let identity = commit_identity(42);
    let derivation_evidence: QueryCommitDigestDerivationEvidence =
        FoundationalIdentityDigestDerivationEvidence::derive_from_authority(
            &identity,
            identity_digest(),
            authority(),
        );
    let evidence: QueryCommitDigestEvidence =
        FoundationalDigestIdentityEvidence::from_derivation_evidence(derivation_evidence);

    assert_eq!(evidence.digest().metadata().entry_count(), 1);
}

#[test]
fn external_token_needs_authority_admission() {
    let token = FoundationalExternalIdentityToken::<u64, QuerySnapshotIdentityKind>::new(77);
    let admitted = token.admit_with_authority::<QueryIdentityAuthority>(authority());
    let identity = FoundationalAuthorityIdentity::from_admitted(admitted);

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
