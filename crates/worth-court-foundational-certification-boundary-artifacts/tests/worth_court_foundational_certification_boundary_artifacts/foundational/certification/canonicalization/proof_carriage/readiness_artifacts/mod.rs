#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::proof_carriage::readiness_artifacts::*;
use {worth_foundational::admit_canonical_sequence_digest_derivation, worth_foundational::certify_canonical_milestone2_production_readiness, worth_foundational::prepare_canonical_basis_bundle, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::prepare_canonical_export_bundle, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadinessProofs, worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadinessProofs, worth_foundational::CanonicalBundleReadyArtifact, worth_foundational::CanonicalComparisonReadinessProofs, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalDigestDerivationReadinessProofs, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalExportReadinessProofs, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalProducerShape, worth_foundational::CanonicalProductionReadinessCertified, worth_foundational::CanonicalProductionTestReadyArtifact, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion};
use {worth_proof::Proof, worth_proof::TransitionOutcome};

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
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_value_sequence(comparison_version.clone(), "same", 1),
        ready_value_sequence(comparison_version, "same", 1),
    ) {
        worth_proof::TransitionOutcome::Success(comparison) => comparison,
        _ => panic!("comparison should be ready"),
    };
    accepts_comparison_proofs(comparison.proofs());

    let export_version = version("m2.proof-carriage.export");
    let export = match prepare_canonical_export_bundle(
        "proof-carriage-export",
        worth_foundational::CanonicalProducerShape::GoldenFixture,
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready_bundle(export_version),
    ) {
        worth_proof::TransitionOutcome::Success(export) => export,
        _ => panic!("export should be ready"),
    };
    accepts_export_proofs(export.proofs());

    let digest_version = version("m2.proof-carriage.digest");
    let digest = match admit_canonical_sequence_digest_derivation(
        ready_value_sequence(digest_version.clone(), "digest", 1),
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            worth_foundational::CanonicalBasisDomain::Value,
            digest_version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(digest) => digest,
        _ => panic!("digest derivation should be ready"),
    };
    accepts_digest_proofs(digest.proofs());

    let production: CanonicalProductionTestReadyArtifact =
        certify_canonical_milestone2_production_readiness();
    accepts_production_readiness_proof(production.proofs());
}
