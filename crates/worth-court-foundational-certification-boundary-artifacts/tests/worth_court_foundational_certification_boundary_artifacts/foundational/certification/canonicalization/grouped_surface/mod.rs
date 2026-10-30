#[allow(unused_imports)]
use crate::certification::ordinary_context::canonicalization::grouped_surface::*;

#[test]
fn grouped_canonicalization_surface_exposes_common_lower_and_stronger_lanes() {
    let left_for_common =
        match worth_foundational::canonicalization_api::common_path::canonicalization()
            .basis()
            .at(version())
            .from_state(admitted_scalar_state("task.count", 10, 8))
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("left basis should be ready"),
        };
    let right_for_common =
        match worth_foundational::canonicalization_api::common_path::canonicalization()
            .basis()
            .at(version())
            .from_state(admitted_scalar_state("task.count", 10, 8))
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("right basis should be ready"),
        };
    let comparison_ready =
        match worth_foundational::canonicalization_api::common_path::canonicalization()
            .compare()
            .left(left_for_common)
            .right(right_for_common)
            .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("comparison should be ready"),
        };
    let common_outcome = worth_foundational::canonicalization_api::common_path::canonicalization()
        .compare()
        .evaluate(&comparison_ready);

    let left_for_lower =
        match worth_foundational::canonicalization_api::common_path::canonicalization()
            .basis()
            .at(version())
            .from_state(admitted_scalar_state("task.count", 10, 8))
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("left lower-lane basis should be ready"),
        };
    let right_for_lower =
        match worth_foundational::canonicalization_api::common_path::canonicalization()
            .basis()
            .at(version())
            .from_state(admitted_scalar_state("task.count", 10, 8))
        {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("right lower-lane basis should be ready"),
        };
    let lower_input = match worth_foundational::canonicalization_api::lower_lane::comparison::prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_for_lower,
        right_for_lower,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane comparison should be ready"),
    };
    let lower_outcome =
        worth_foundational::canonicalization_api::lower_lane::comparison::compare_canonical_basis(
            &lower_input,
        );
    let lower_bundle = match worth_foundational::canonicalization_api::lower_lane::basis::prepare_canonical_basis_bundle(
        version(),
        [
            match worth_foundational::canonicalization_api::common_path::canonicalization()
                .basis()
                .at(version())
                .from_state(admitted_scalar_state("task.count", 10, 8))
            {
                worth_proof::TransitionOutcome::Success(ready) => ready,
                _ => panic!("lower-lane bundle basis should be ready"),
            },
        ],
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane bundle should be ready"),
    };
    let lower_export_ready_for_compare = match worth_foundational::canonicalization_api::lower_lane::export::prepare_canonical_export_bundle(
        "grouped-surface",
        worth_foundational::CanonicalProducerShape::GoldenFixture,
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        match worth_foundational::canonicalization_api::lower_lane::basis::prepare_canonical_basis_bundle(
            version(),
            [
                match worth_foundational::canonicalization_api::common_path::canonicalization()
                    .basis()
                    .at(version())
                    .from_state(admitted_scalar_state("task.count", 10, 8))
                {
                    worth_proof::TransitionOutcome::Success(ready) => ready,
                    _ => panic!("lower-lane compare bundle basis should be ready"),
                },
            ],
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("lower-lane compare bundle should be ready"),
        },
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane export should be ready for comparison"),
    };
    let lower_export_outcome =
        worth_foundational::canonicalization_api::lower_lane::export::compare_canonical_exports(
            &lower_export_ready_for_compare,
            &lower_export_ready_for_compare,
        );
    let lower_export_ready_for_digest = match worth_foundational::canonicalization_api::lower_lane::export::prepare_canonical_export_bundle(
        "grouped-surface",
        worth_foundational::CanonicalProducerShape::GoldenFixture,
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        lower_bundle,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane export should be ready for digest"),
    };
    let lower_digest_ready = match worth_foundational::canonicalization_api::lower_lane::digest::admit_canonical_export_digest_derivation(
        lower_export_ready_for_digest,
        worth_foundational::canonicalization_api::lower_lane::digest::CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            worth_foundational::canonicalization_api::lower_lane::digest::CanonicalDigestAlgorithmId::sha256(),
            version(),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("lower-lane digest derivation should be ready"),
    };
    let lower_digest =
        worth_foundational::canonicalization_api::lower_lane::digest::derive_canonical_digest(
            lower_digest_ready,
        );
    let report = worth_foundational::canonicalization_api::stronger_lane::readiness::canonical_milestone2_production_readiness_report();
    let certified = worth_foundational::canonicalization_api::stronger_lane::readiness::certify_canonical_milestone2_production_readiness();

    assert!(matches!(
        common_outcome,
        worth_foundational::canonicalization_api::lower_lane::comparison::CanonicalComparisonOutcome::Equivalent(_)
    ));
    assert!(matches!(
        lower_outcome,
        worth_foundational::canonicalization_api::lower_lane::comparison::CanonicalComparisonOutcome::Equivalent(_)
    ));
    assert!(matches!(
        lower_export_outcome,
        worth_foundational::canonicalization_api::lower_lane::export::CanonicalExportComparisonOutcome::Equivalent
    ));
    assert_eq!(lower_digest.metadata().entry_count(), 2);
    assert!(report.passes_readiness_checklist());
    assert!(report
        .phase_gates()
        .iter()
        .any(|gate| gate.gate() == worth_foundational::CanonicalMilestone2PhaseGate::DigestSlots));
    assert!(std::ptr::eq(
        worth_foundational::canonicalization_api::stronger_lane::readiness::require_canonical_production_test_readiness(&certified),
        certified.payload()
    ));
}
