use worth_foundational::{
    admit_authoritative_record_aspect_state,
    canonicalization_api::{
        common_path,
        lower_lane::{
            basis as lower_basis, comparison as lower_comparison, digest as lower_digest,
            export as lower_export,
        },
        stronger_lane::readiness as stronger_readiness,
    },
    validate_aspect_value, AspectValue, CanonicalEquivalenceBasis, CanonicalMilestone2PhaseGate,
    CanonicalProducerShape, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::scalar_contract;

fn version() -> lower_basis::CanonicalizationRuleVersion {
    lower_basis::CanonicalizationRuleVersion::new("m2.dx.grouped-surface").expect("valid version")
}

fn admitted_scalar_state(
    aspect_key: &str,
    aspect_identity: u64,
    value: i64,
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let contract = scalar_contract(aspect_key, aspect_identity, ScalarAspectType::Int64);
    let TransitionOutcome::Success(validated) =
        validate_aspect_value(&contract, AspectValue::Int64(value).into())
    else {
        panic!("expected validation");
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([validated])
    else {
        panic!("expected authoritative state");
    };
    state
}

#[test]
fn grouped_canonicalization_surface_exposes_common_lower_and_stronger_lanes() {
    let left_for_common = match common_path::canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 8))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("left basis should be ready"),
    };
    let right_for_common = match common_path::canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 8))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("right basis should be ready"),
    };
    let comparison_ready = match common_path::canonicalization()
        .compare()
        .left(left_for_common)
        .right(right_for_common)
        .under(CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };
    let common_outcome = common_path::canonicalization()
        .compare()
        .evaluate(&comparison_ready);

    let left_for_lower = match common_path::canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 8))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("left lower-lane basis should be ready"),
    };
    let right_for_lower = match common_path::canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 8))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("right lower-lane basis should be ready"),
    };
    let lower_input = match lower_comparison::prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_for_lower,
        right_for_lower,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane comparison should be ready"),
    };
    let lower_outcome = lower_comparison::compare_canonical_basis(&lower_input);
    let lower_bundle = match lower_basis::prepare_canonical_basis_bundle(
        version(),
        [
            match common_path::canonicalization()
                .basis()
                .at(version())
                .from_state(admitted_scalar_state("task.count", 10, 8))
            {
                TransitionOutcome::Success(ready) => ready,
                _ => panic!("lower-lane bundle basis should be ready"),
            },
        ],
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane bundle should be ready"),
    };
    let lower_export_ready_for_compare = match lower_export::prepare_canonical_export_bundle(
        "grouped-surface",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        match lower_basis::prepare_canonical_basis_bundle(
            version(),
            [
                match common_path::canonicalization()
                    .basis()
                    .at(version())
                    .from_state(admitted_scalar_state("task.count", 10, 8))
                {
                    TransitionOutcome::Success(ready) => ready,
                    _ => panic!("lower-lane compare bundle basis should be ready"),
                },
            ],
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("lower-lane compare bundle should be ready"),
        },
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane export should be ready for comparison"),
    };
    let lower_export_outcome = lower_export::compare_canonical_exports(
        &lower_export_ready_for_compare,
        &lower_export_ready_for_compare,
    );
    let lower_export_ready_for_digest = match lower_export::prepare_canonical_export_bundle(
        "grouped-surface",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        lower_bundle,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane export should be ready for digest"),
    };
    let lower_digest_ready = match lower_digest::admit_canonical_export_digest_derivation(
        lower_export_ready_for_digest,
        lower_digest::CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            lower_digest::CanonicalDigestAlgorithmId::sha256(),
            version(),
        ),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane digest derivation should be ready"),
    };
    let lower_digest = lower_digest::derive_canonical_digest(lower_digest_ready);
    let report = stronger_readiness::canonical_milestone2_production_readiness_report();
    let certified = stronger_readiness::certify_canonical_milestone2_production_readiness();

    assert!(matches!(
        common_outcome,
        lower_comparison::CanonicalComparisonOutcome::Equivalent(_)
    ));
    assert!(matches!(
        lower_outcome,
        lower_comparison::CanonicalComparisonOutcome::Equivalent(_)
    ));
    assert!(matches!(
        lower_export_outcome,
        lower_export::CanonicalExportComparisonOutcome::Equivalent
    ));
    assert_eq!(lower_digest.metadata().entry_count(), 2);
    assert!(report.passes_readiness_checklist());
    assert!(report
        .phase_gates()
        .iter()
        .any(|gate| gate.gate() == CanonicalMilestone2PhaseGate::DigestSlots));
    assert!(std::ptr::eq(
        stronger_readiness::require_canonical_production_test_readiness(&certified),
        certified.payload()
    ));
}
