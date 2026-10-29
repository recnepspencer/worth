use worth_foundational::{
    admit_canonical_bundle_digest_derivation, admit_canonical_export_digest_derivation,
    admit_canonical_sequence_digest_derivation, compare_canonical_basis, derive_canonical_digest,
    prepare_canonical_basis_bundle, prepare_canonical_basis_sequence, prepare_canonical_comparison,
    prepare_canonical_export_bundle, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalBundleReadyArtifact, CanonicalComparisonOutcome, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestDerivationReadyArtifact,
    CanonicalDigestInputDomain, CanonicalDigestInputShape,
    CanonicalDomainBundleDigestAlgorithmSlot, CanonicalEquivalenceBasis,
    CanonicalExportBundleDigestAlgorithmSlot, CanonicalIntegerWidth, CanonicalProducerShape,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

fn signed_value_entry(
    domain: CanonicalBasisDomain,
    label: &str,
    value: i64,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(label.into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

fn identity_entry(label: &str, value: u64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Identity,
        CanonicalBasisLocus::Named(label.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn ready_sequence(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(version, domain, entries) {
        TransitionOutcome::Success(sequence) => sequence,
        _ => panic!("basis sequence should be ready"),
    }
}

fn ready_bundle(version: CanonicalizationRuleVersion) -> CanonicalBundleReadyArtifact {
    let value = ready_sequence(
        version.clone(),
        CanonicalBasisDomain::Value,
        [
            signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1),
            signed_value_entry(CanonicalBasisDomain::Value, "zeta", 2),
        ],
    );
    let identity = ready_sequence(
        version.clone(),
        CanonicalBasisDomain::Identity,
        [identity_entry("handle", 7)],
    );

    match prepare_canonical_basis_bundle(version, [identity, value]) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("basis bundle should be ready"),
    }
}

fn accepts_digest_derivation_ready(_: &CanonicalDigestDerivationReadyArtifact) {}

#[test]
fn sha256_sequence_digest_records_exact_algorithm_metadata() {
    let version = version("m2.phase5.sha256");
    let ready = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            CanonicalBasisDomain::Value,
            [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
        ),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Value,
            version,
        ),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("SHA-256 digest derivation should be admitted"),
    };

    accepts_digest_derivation_ready(&ready);
    let digest = derive_canonical_digest(ready);

    assert_eq!(
        digest.metadata().algorithm().id(),
        &CanonicalDigestAlgorithmId::sha256()
    );
    assert_eq!(digest.value().bytes().len(), 32);
}

#[test]
fn sequence_digest_derivation_records_algorithm_version_domain_and_entry_count() {
    let version = version("m2.phase5.sequence");
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version.clone(),
    );
    let ready = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version,
            CanonicalBasisDomain::Value,
            [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
        ),
        slot,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest derivation should be admitted"),
    };

    accepts_digest_derivation_ready(&ready);
    let digest = derive_canonical_digest(ready);

    assert_eq!(
        digest.metadata().algorithm().id(),
        &CanonicalDigestAlgorithmId::sha256()
    );
    assert_eq!(
        digest.metadata().algorithm().input_domain(),
        CanonicalDigestInputDomain::Single(CanonicalBasisDomain::Value)
    );
    assert_eq!(
        digest.metadata().algorithm().input_shape(),
        CanonicalDigestInputShape::SingleSequence
    );
    assert_eq!(digest.metadata().entry_count(), 1);
    assert_eq!(digest.value().bytes().len(), 32);
}

#[test]
fn bundle_and_export_digest_slots_have_distinct_input_shape_metadata() {
    let version = version("m2.phase5.bundle");
    let bundle_digest = match admit_canonical_bundle_digest_derivation(
        ready_bundle(version.clone()),
        CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
            CanonicalDigestAlgorithmId::sha256(),
            version.clone(),
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("domain bundle digest should be admitted"),
    };
    let export = match prepare_canonical_export_bundle(
        "digest-export",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version.clone()),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };
    let export_digest = match admit_canonical_export_digest_derivation(
        export,
        CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            CanonicalDigestAlgorithmId::sha256(),
            version,
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("export digest should be admitted"),
    };

    assert_eq!(
        bundle_digest.metadata().algorithm().input_shape(),
        CanonicalDigestInputShape::DomainBundle
    );
    assert_eq!(
        export_digest.metadata().algorithm().input_shape(),
        CanonicalDigestInputShape::ExportBundle
    );
    assert_ne!(bundle_digest.value(), export_digest.value());
}

#[test]
fn algorithm_slot_admission_denies_unsupported_version_domain_and_algorithm() {
    let sequence = ready_sequence(
        version("m2.phase5.admission"),
        CanonicalBasisDomain::Value,
        [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
    );
    let wrong_version = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version("m2.phase5.other"),
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, wrong_version),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::RuleVersionMismatch)
    ));

    let wrong_domain = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Identity,
        version("m2.phase5.domain"),
    );
    let sequence = ready_sequence(
        version("m2.phase5.domain"),
        CanonicalBasisDomain::Value,
        [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, wrong_domain),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::InputDomainMismatch)
    ));

    let unsupported_algorithm = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::new("example.sha999").expect("valid id"),
        CanonicalBasisDomain::Value,
        version("m2.phase5.algorithm"),
    );
    let sequence = ready_sequence(
        version("m2.phase5.algorithm"),
        CanonicalBasisDomain::Value,
        [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, unsupported_algorithm),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::UnsupportedAlgorithm)
    ));
}

#[test]
fn equal_looking_storage_values_in_different_domains_produce_distinct_digests() {
    let version = version("m2.phase5.collision");
    let value_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            CanonicalBasisDomain::Value,
            [signed_value_entry(CanonicalBasisDomain::Value, "same", 9)],
        ),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Value,
            version.clone(),
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("value digest should be admitted"),
    };
    let identity_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            CanonicalBasisDomain::Identity,
            [signed_value_entry(
                CanonicalBasisDomain::Identity,
                "same",
                9,
            )],
        ),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Identity,
            version,
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("identity digest should be admitted"),
    };

    assert_ne!(value_digest.value(), identity_digest.value());
}

#[test]
fn delimiter_shaped_future_domain_tokens_remain_distinct_digest_inputs() {
    let version = version("m2.phase5.delimiter-shaped-domain");
    let left_domain = CanonicalBasisDomain::Future("future:alpha|beta;value");
    let right_domain = CanonicalBasisDomain::Future("future:alpha|beta");
    let left_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            left_domain,
            [signed_value_entry(left_domain, "same|locus:1", 9)],
        ),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            left_domain,
            version.clone(),
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("left delimiter-shaped digest should be admitted"),
    };
    let right_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            right_domain,
            [signed_value_entry(right_domain, "same|locus:1;value", 9)],
        ),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            right_domain,
            version,
        ),
    ) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("right delimiter-shaped digest should be admitted"),
    };

    assert_ne!(left_digest.value(), right_digest.value());
    assert_ne!(
        left_digest.metadata().input_id(),
        right_digest.metadata().input_id()
    );
}

#[test]
fn matching_digest_values_do_not_replace_explicit_equivalence_basis() {
    let version = version("m2.phase5.equivalence");
    let left = ready_sequence(
        version.clone(),
        CanonicalBasisDomain::Value,
        [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
    );
    let right = ready_sequence(
        version,
        CanonicalBasisDomain::Value,
        [signed_value_entry(CanonicalBasisDomain::Value, "alpha", 1)],
    );

    match prepare_canonical_comparison(CanonicalEquivalenceBasis::DigestEquivalent, left, right) {
        TransitionOutcome::Success(ready) => {
            assert!(matches!(
                compare_canonical_basis(&ready),
                CanonicalComparisonOutcome::Unsupported(_)
            ));
        }
        _ => panic!("comparison readiness should carry explicit digest-equivalent basis"),
    }
}
