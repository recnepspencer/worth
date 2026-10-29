#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::digest_slots::digest_derivation::*;
use {worth_foundational::admit_canonical_bundle_digest_derivation, worth_foundational::admit_canonical_export_digest_derivation, worth_foundational::admit_canonical_sequence_digest_derivation, worth_foundational::compare_canonical_basis, worth_foundational::derive_canonical_digest, worth_foundational::prepare_canonical_basis_bundle, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::prepare_canonical_export_bundle, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadyArtifact, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalDigestDerivationDenial, worth_foundational::CanonicalDigestDerivationReadyArtifact, worth_foundational::CanonicalDigestInputDomain, worth_foundational::CanonicalDigestInputShape, worth_foundational::CanonicalDomainBundleDigestAlgorithmSlot, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalExportBundleDigestAlgorithmSlot, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalProducerShape, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion};
use {worth_proof::TransitionOutcome};

#[test]
fn sha256_sequence_digest_records_exact_algorithm_metadata() {
    let version = version("m2.phase5.sha256");
    let ready = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            worth_foundational::CanonicalBasisDomain::Value,
            [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
        ),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            worth_foundational::CanonicalBasisDomain::Value,
            version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("SHA-256 digest derivation should be admitted"),
    };

    accepts_digest_derivation_ready(&ready);
    let digest = derive_canonical_digest(ready);

    assert_eq!(
        digest.metadata().algorithm().id(),
        &worth_foundational::CanonicalDigestAlgorithmId::sha256()
    );
    assert_eq!(digest.value().bytes().len(), 32);
}

#[test]
fn sequence_digest_derivation_records_algorithm_version_domain_and_entry_count() {
    let version = version("m2.phase5.sequence");
    let slot = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Value,
        version.clone(),
    );
    let ready = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version,
            worth_foundational::CanonicalBasisDomain::Value,
            [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
        ),
        slot,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest derivation should be admitted"),
    };

    accepts_digest_derivation_ready(&ready);
    let digest = derive_canonical_digest(ready);

    assert_eq!(
        digest.metadata().algorithm().id(),
        &worth_foundational::CanonicalDigestAlgorithmId::sha256()
    );
    assert_eq!(
        digest.metadata().algorithm().input_domain(),
        worth_foundational::CanonicalDigestInputDomain::Single(worth_foundational::CanonicalBasisDomain::Value)
    );
    assert_eq!(
        digest.metadata().algorithm().input_shape(),
        worth_foundational::CanonicalDigestInputShape::SingleSequence
    );
    assert_eq!(digest.metadata().entry_count(), 1);
    assert_eq!(digest.value().bytes().len(), 32);
}

#[test]
fn bundle_and_export_digest_slots_have_distinct_input_shape_metadata() {
    let version = version("m2.phase5.bundle");
    let bundle_digest = match admit_canonical_bundle_digest_derivation(
        ready_bundle(version.clone()),
        worth_foundational::CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            version.clone(),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("domain bundle digest should be admitted"),
    };
    let export = match prepare_canonical_export_bundle(
        "digest-export",
        worth_foundational::CanonicalProducerShape::GoldenFixture,
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(version.clone()),
    ) {
        worth_proof::TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };
    let export_digest = match admit_canonical_export_digest_derivation(
        export,
        worth_foundational::CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("export digest should be admitted"),
    };

    assert_eq!(
        bundle_digest.metadata().algorithm().input_shape(),
        worth_foundational::CanonicalDigestInputShape::DomainBundle
    );
    assert_eq!(
        export_digest.metadata().algorithm().input_shape(),
        worth_foundational::CanonicalDigestInputShape::ExportBundle
    );
    assert_ne!(bundle_digest.value(), export_digest.value());
}

#[test]
fn algorithm_slot_admission_denies_unsupported_version_domain_and_algorithm() {
    let sequence = ready_sequence(
        version("m2.phase5.admission"),
        worth_foundational::CanonicalBasisDomain::Value,
        [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
    );
    let wrong_version = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Value,
        version("m2.phase5.other"),
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, wrong_version),
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalDigestDerivationDenial::RuleVersionMismatch)
    ));

    let wrong_domain = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Identity,
        version("m2.phase5.domain"),
    );
    let sequence = ready_sequence(
        version("m2.phase5.domain"),
        worth_foundational::CanonicalBasisDomain::Value,
        [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, wrong_domain),
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalDigestDerivationDenial::InputDomainMismatch)
    ));

    let unsupported_algorithm = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::new("example.sha999").expect("valid id"),
        worth_foundational::CanonicalBasisDomain::Value,
        version("m2.phase5.algorithm"),
    );
    let sequence = ready_sequence(
        version("m2.phase5.algorithm"),
        worth_foundational::CanonicalBasisDomain::Value,
        [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
    );

    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence, unsupported_algorithm),
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalDigestDerivationDenial::UnsupportedAlgorithm)
    ));
}

#[test]
fn equal_looking_storage_values_in_different_domains_produce_distinct_digests() {
    let version = version("m2.phase5.collision");
    let value_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            worth_foundational::CanonicalBasisDomain::Value,
            [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "same", 9)],
        ),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            worth_foundational::CanonicalBasisDomain::Value,
            version.clone(),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("value digest should be admitted"),
    };
    let identity_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            worth_foundational::CanonicalBasisDomain::Identity,
            [signed_value_entry(
                worth_foundational::CanonicalBasisDomain::Identity,
                "same",
                9,
            )],
        ),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            worth_foundational::CanonicalBasisDomain::Identity,
            version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("identity digest should be admitted"),
    };

    assert_ne!(value_digest.value(), identity_digest.value());
}

#[test]
fn delimiter_shaped_future_domain_tokens_remain_distinct_digest_inputs() {
    let version = version("m2.phase5.delimiter-shaped-domain");
    let left_domain = worth_foundational::CanonicalBasisDomain::Future("future:alpha|beta;value");
    let right_domain = worth_foundational::CanonicalBasisDomain::Future("future:alpha|beta");
    let left_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            left_domain,
            [signed_value_entry(left_domain, "same|locus:1", 9)],
        ),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            left_domain,
            version.clone(),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        _ => panic!("left delimiter-shaped digest should be admitted"),
    };
    let right_digest = match admit_canonical_sequence_digest_derivation(
        ready_sequence(
            version.clone(),
            right_domain,
            [signed_value_entry(right_domain, "same|locus:1;value", 9)],
        ),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            right_domain,
            version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
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
        worth_foundational::CanonicalBasisDomain::Value,
        [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
    );
    let right = ready_sequence(
        version,
        worth_foundational::CanonicalBasisDomain::Value,
        [signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1)],
    );

    match prepare_canonical_comparison(worth_foundational::CanonicalEquivalenceBasis::DigestEquivalent, left, right) {
        worth_proof::TransitionOutcome::Success(ready) => {
            assert!(matches!(
                compare_canonical_basis(&ready),
                worth_foundational::CanonicalComparisonOutcome::Unsupported(_)
            ));
        }
        _ => panic!("comparison readiness should carry explicit digest-equivalent basis"),
    }
}
