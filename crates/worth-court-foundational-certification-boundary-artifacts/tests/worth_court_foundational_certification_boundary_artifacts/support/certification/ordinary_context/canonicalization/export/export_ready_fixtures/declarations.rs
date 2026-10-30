pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::bridge_canonical_export_trust_boundary,
    worth_foundational::compare_canonical_exports,
    worth_foundational::prepare_canonical_basis_bundle,
    worth_foundational::prepare_canonical_basis_sequence,
    worth_foundational::prepare_canonical_export_bundle, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadyArtifact,
    worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalExportComparisonOutcome, worth_foundational::CanonicalExportDebt,
    worth_foundational::CanonicalExportManifestMismatchKind,
    worth_foundational::CanonicalExportReadinessProofs, worth_foundational::CanonicalIntegerWidth,
    worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalProducerShape,
    worth_foundational::CanonicalizationRuleVersion,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub(crate) fn value_entry(label: &str, value: i64) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Value,
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
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis sequence should be ready"),
    }
}

pub(crate) fn ready_bundle(
    version: CanonicalizationRuleVersion,
    value_entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBundleReadyArtifact {
    let value = ready_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Value,
        value_entries,
    );
    let identity = ready_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Identity,
        [identity_entry("boundary.handle", 77)],
    );

    match prepare_canonical_basis_bundle(version, [identity, value]) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("basis bundle should be ready"),
    }
}

pub(crate) fn accepts_export_readiness_proofs(_: &CanonicalExportReadinessProofs) {}
