pub(crate) use {worth_foundational::admit_canonical_sequence_digest_derivation, worth_foundational::certify_canonical_milestone2_production_readiness, worth_foundational::prepare_canonical_basis_bundle, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::prepare_canonical_export_bundle, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadinessProofs, worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadinessProofs, worth_foundational::CanonicalBundleReadyArtifact, worth_foundational::CanonicalComparisonReadinessProofs, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalDigestDerivationReadinessProofs, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalExportReadinessProofs, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalProducerShape, worth_foundational::CanonicalProductionReadinessCertified, worth_foundational::CanonicalProductionTestReadyArtifact, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion};
pub(crate) use {worth_proof::Proof, worth_proof::TransitionOutcome};

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

pub(crate) fn ready_value_sequence(
    version: CanonicalizationRuleVersion,
    label: &str,
    value: i64,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry(label, value)],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("value sequence should be ready"),
    }
}

pub(crate) fn ready_identity_sequence(version: CanonicalizationRuleVersion) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        worth_foundational::CanonicalBasisDomain::Identity,
        [identity_entry("identity", 7)],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity sequence should be ready"),
    }
}

pub(crate) fn ready_bundle(version: CanonicalizationRuleVersion) -> CanonicalBundleReadyArtifact {
    match prepare_canonical_basis_bundle(
        version.clone(),
        [
            ready_value_sequence(version.clone(), "value", 1),
            ready_identity_sequence(version),
        ],
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("bundle should be ready"),
    }
}

pub(crate) fn accepts_basis_proofs(_: &CanonicalBasisReadinessProofs) {}
pub(crate) fn accepts_bundle_proofs(_: &CanonicalBundleReadinessProofs) {}
pub(crate) fn accepts_comparison_proofs(_: &CanonicalComparisonReadinessProofs) {}
pub(crate) fn accepts_export_proofs(_: &CanonicalExportReadinessProofs) {}
pub(crate) fn accepts_digest_proofs(_: &CanonicalDigestDerivationReadinessProofs) {}
pub(crate) fn accepts_production_readiness_proof(
    _: &Proof<
        CanonicalProductionReadinessCertified,
        worth_foundational::CanonicalProductionReadinessAuthority,
    >,
) {
}