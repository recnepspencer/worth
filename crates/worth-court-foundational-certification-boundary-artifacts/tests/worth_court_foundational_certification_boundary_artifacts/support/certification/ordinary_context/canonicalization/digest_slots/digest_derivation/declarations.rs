pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::admit_canonical_bundle_digest_derivation,
    worth_foundational::admit_canonical_export_digest_derivation,
    worth_foundational::admit_canonical_sequence_digest_derivation,
    worth_foundational::compare_canonical_basis, worth_foundational::derive_canonical_digest,
    worth_foundational::prepare_canonical_basis_bundle,
    worth_foundational::prepare_canonical_basis_sequence,
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::prepare_canonical_export_bundle, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadyArtifact,
    worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalDigestAlgorithmId,
    worth_foundational::CanonicalDigestDerivationDenial,
    worth_foundational::CanonicalDigestDerivationReadyArtifact,
    worth_foundational::CanonicalDigestInputDomain, worth_foundational::CanonicalDigestInputShape,
    worth_foundational::CanonicalDomainBundleDigestAlgorithmSlot,
    worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalExportBundleDigestAlgorithmSlot,
    worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalProducerShape,
    worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot,
    worth_foundational::CanonicalizationRuleVersion,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub(crate) fn signed_value_entry(
    domain: CanonicalBasisDomain,
    label: &str,
    value: i64,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        domain,
        worth_foundational::CanonicalBasisLocus::Named(label.into()),
        worth_foundational::CanonicalBasisEntryKind::Value,
        worth_foundational::CanonicalBasisValue::SignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

pub(crate) fn identity_entry(label: &str, value: u64) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Identity,
        worth_foundational::CanonicalBasisLocus::Named(label.into()),
        worth_foundational::CanonicalBasisEntryKind::Identity,
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

pub(crate) fn ready_sequence(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(version, domain, entries) {
        worth_proof::TransitionOutcome::Success(sequence) => sequence,
        _ => panic!("basis sequence should be ready"),
    }
}

pub(crate) fn ready_bundle(version: CanonicalizationRuleVersion) -> CanonicalBundleReadyArtifact {
    let value = ready_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Value,
        [
            signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "alpha", 1),
            signed_value_entry(worth_foundational::CanonicalBasisDomain::Value, "zeta", 2),
        ],
    );
    let identity = ready_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Identity,
        [identity_entry("handle", 7)],
    );

    match prepare_canonical_basis_bundle(version, [identity, value]) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("basis bundle should be ready"),
    }
}

pub(crate) fn accepts_digest_derivation_ready(_: &CanonicalDigestDerivationReadyArtifact) {}
