pub(crate) use {worth_foundational::admit_canonical_sequence_digest_derivation, worth_foundational::admit_foundational_authority_identity, worth_foundational::admit_foundational_external_identity_token, worth_foundational::derive_canonical_digest, worth_foundational::derive_foundational_digest_identity_evidence, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::project_foundational_identity, worth_foundational::readmit_foundational_authority_identity, worth_foundational::readmit_revalidated_foundational_authority_identity, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion, worth_foundational::FoundationalAdmittedIdentityValue, worth_foundational::FoundationalAuthorityIdentity, worth_foundational::FoundationalDigestIdentityEvidence, worth_foundational::FoundationalExternalIdentityToken, worth_foundational::FoundationalIdentityBasis, worth_foundational::FoundationalIdentityDigestDerivationEvidence, worth_foundational::FoundationalIdentityKind, worth_foundational::FoundationalIdentityProjectionEvidence, worth_foundational::FoundationalProjectionIdentity};
pub(crate) use {worth_proof::AuthorityMarker, worth_proof::AuthorityWitness, worth_proof::TransitionOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct QueryIdentityAuthority(pub(crate) ());
impl AuthorityMarker for QueryIdentityAuthority {}

pub(crate) struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

pub(crate) struct QuerySnapshotIdentityKind;
impl FoundationalIdentityKind for QuerySnapshotIdentityKind {}

pub(crate) struct QueryCommitIdentityBasis;
impl FoundationalIdentityBasis for QueryCommitIdentityBasis {}

pub(crate) type QueryCommitIdentity =
    FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>;

pub(crate) type QueryCommitAdmittedValue =
    FoundationalAdmittedIdentityValue<u64, QueryIdentityAuthority, QueryCommitIdentityKind>;

pub(crate) type QueryCommitProjectionEvidence =
    FoundationalIdentityProjectionEvidence<String, QueryIdentityAuthority, QueryCommitIdentityKind>;

pub(crate) type QueryCommitDigestDerivationEvidence = FoundationalIdentityDigestDerivationEvidence<
    QueryCommitIdentityBasis,
    QueryIdentityAuthority,
    QueryCommitIdentityKind,
>;

pub(crate) type QueryCommitDigestEvidence = FoundationalDigestIdentityEvidence<
    QueryCommitIdentityBasis,
    QueryIdentityAuthority,
    QueryCommitIdentityKind,
>;

pub(crate) fn authority() -> AuthorityWitness<QueryIdentityAuthority> {
    worth_proof::AuthorityWitness::from_authority_marker(QueryIdentityAuthority(()))
}

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub(crate) fn identity_digest() -> worth_foundational::CanonicalDerivedDigest {
    let version = version("authority.identity.boundary");
    let entry = worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Identity,
        worth_foundational::CanonicalBasisLocus::Named("query.commit".into()),
        worth_foundational::CanonicalBasisEntryKind::Identity,
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: 42,
        },
    );
    let sequence = match prepare_canonical_basis_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Identity,
        [entry],
    ) {
        worth_proof::TransitionOutcome::Success(sequence) => sequence,
        _ => panic!("identity basis should be ready"),
    };
    let slot = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Identity,
        version,
    );
    match admit_canonical_sequence_digest_derivation(sequence, slot) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("identity digest derivation should be admitted"),
    }
}

pub(crate) fn admitted_commit_identity_value(value: u64) -> QueryCommitAdmittedValue {
    worth_foundational::FoundationalAdmittedIdentityValue::admit(value, authority())
}

pub(crate) fn commit_identity(value: u64) -> QueryCommitIdentity {
    crate::certification::ordinary_context::identities::authority_identity_boundaries::QueryCommitIdentity::from_admitted(admitted_commit_identity_value(value))
}