use worth_foundational::{
    admit_canonical_sequence_digest_derivation, certify_canonical_milestone2_production_readiness,
    prepare_canonical_basis_bundle, prepare_canonical_basis_sequence, prepare_canonical_comparison,
    prepare_canonical_export_bundle, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadinessProofs,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalBundleReadinessProofs,
    CanonicalBundleReadyArtifact, CanonicalComparisonReadinessProofs, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationReadinessProofs, CanonicalEquivalenceBasis,
    CanonicalExportReadinessProofs, CanonicalIntegerWidth, CanonicalProducerShape,
    CanonicalProductionReadinessCertified, CanonicalProductionTestReadyArtifact,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion,
};
use worth_proof::{Proof, TransitionOutcome};

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

fn value_entry(label: &str, value: i64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
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

fn ready_value_sequence(
    version: CanonicalizationRuleVersion,
    label: &str,
    value: i64,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Value,
        [value_entry(label, value)],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("value sequence should be ready"),
    }
}

fn ready_identity_sequence(version: CanonicalizationRuleVersion) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Identity,
        [identity_entry("identity", 7)],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("identity sequence should be ready"),
    }
}

fn ready_bundle(version: CanonicalizationRuleVersion) -> CanonicalBundleReadyArtifact {
    match prepare_canonical_basis_bundle(
        version.clone(),
        [
            ready_value_sequence(version.clone(), "value", 1),
            ready_identity_sequence(version),
        ],
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("bundle should be ready"),
    }
}

fn accepts_basis_proofs(_: &CanonicalBasisReadinessProofs) {}
fn accepts_bundle_proofs(_: &CanonicalBundleReadinessProofs) {}
fn accepts_comparison_proofs(_: &CanonicalComparisonReadinessProofs) {}
fn accepts_export_proofs(_: &CanonicalExportReadinessProofs) {}
fn accepts_digest_proofs(_: &CanonicalDigestDerivationReadinessProofs) {}
fn accepts_production_readiness_proof(
    _: &Proof<
        CanonicalProductionReadinessCertified,
        worth_foundational::CanonicalProductionReadinessAuthority,
    >,
) {
}

#[test]
fn canonical_basis_and_bundle_artifacts_carry_named_readiness_proof_sets() {
    let version = version("m2.proof-carriage.basis");
    let sequence = ready_value_sequence(version.clone(), "value", 1);
    let bundle = ready_bundle(version);

    accepts_basis_proofs(sequence.proofs());
    accepts_bundle_proofs(bundle.proofs());
}

#[test]
fn canonical_ready_bundle_artifacts_clone_without_losing_payload_or_proofs() {
    let version = version("m2.proof-carriage.bundle-clone");
    let bundle = ready_bundle(version);
    let cloned = bundle.clone();

    accepts_bundle_proofs(cloned.proofs());
    assert_eq!(bundle.payload().version(), cloned.payload().version());
    assert_eq!(
        bundle.payload().sequences().len(),
        cloned.payload().sequences().len()
    );
}

#[test]
fn comparison_export_digest_and_production_artifacts_carry_named_readiness_proof_sets() {
    let comparison_version = version("m2.proof-carriage.comparison");
    let comparison = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_value_sequence(comparison_version.clone(), "same", 1),
        ready_value_sequence(comparison_version, "same", 1),
    ) {
        TransitionOutcome::Success(comparison) => comparison,
        _ => panic!("comparison should be ready"),
    };
    accepts_comparison_proofs(comparison.proofs());

    let export_version = version("m2.proof-carriage.export");
    let export = match prepare_canonical_export_bundle(
        "proof-carriage-export",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(export_version),
    ) {
        TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };
    accepts_export_proofs(export.proofs());

    let digest_version = version("m2.proof-carriage.digest");
    let digest = match admit_canonical_sequence_digest_derivation(
        ready_value_sequence(digest_version.clone(), "digest", 1),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Value,
            digest_version,
        ),
    ) {
        TransitionOutcome::Success(digest) => digest,
        _ => panic!("digest derivation should be ready"),
    };
    accepts_digest_proofs(digest.proofs());

    let production: CanonicalProductionTestReadyArtifact =
        certify_canonical_milestone2_production_readiness();
    accepts_production_readiness_proof(production.proofs());
}
