#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::front_doors::*;
use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::canonicalization, worth_foundational::validate_aspect_value, worth_foundational::AspectLocator, worth_foundational::AspectValue, worth_foundational::AspectValueLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMilestone2PhaseGate, worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalProducerShape, worth_foundational::CanonicalizationRuleVersion, worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};

use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract};

#[test]
fn canonicalization_front_door_exposes_all_five_milestone2_lanes() {
    let front = canonicalization();
    let left_for_comparison = match front
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 1))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left basis should be ready"),
    };
    let right_for_comparison = match front
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 1))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right basis should be ready"),
    };

    let comparison = match front
        .compare()
        .left(left_for_comparison)
        .right(right_for_comparison)
        .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };
    assert!(matches!(
        front.compare().evaluate(&comparison),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));

    let left_for_bundle = match front
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 1))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("bundle basis should be ready"),
    };
    let bundle = match front.basis().at(version()).bundle([left_for_bundle]) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("bundle should be ready"),
    };
    let export = match front
        .export()
        .from_bundle(bundle)
        .named("front-door")
        .for_producer_shape(worth_foundational::CanonicalProducerShape::GoldenFixture)
        .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("export should be ready"),
    };
    let left_for_digest = match front
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 1))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest basis should be ready"),
    };
    let digest_ready = match front.digest().for_sequence(
        left_for_digest,
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest derivation should be ready"),
    };
    let digest = front.digest().derive(digest_ready);
    let report = front.readiness().report();
    let certified = front.readiness().certify();

    assert_eq!(export.payload().bundle().sequences().len(), 1);
    assert_eq!(digest.metadata().entry_count(), 2);
    assert!(report.passes_readiness_checklist());
    assert!(std::ptr::eq(
        front.readiness().require(&certified),
        certified.payload()
    ));
}

#[test]
fn canonicalization_basis_front_door_prepares_state_and_locator_surfaces() {
    let state = admitted_scalar_state("task.count", 10, 4);

    let state_ready = match canonicalization().basis().at(version()).from_state(state) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("state basis should be ready"),
    };
    let locator_ready = match canonicalization()
        .basis()
        .at(version())
        .from_source_locator(worth_foundational::BoundarySourceLocator::aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::SupportOnly,
            key("task.count"),
        ))) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("locator basis should be ready"),
    };

    assert_eq!(
        state_ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::AuthoritativeState
    );
    assert!(state_ready.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("task.count.value".into())
            && entry.value()
                == &worth_foundational::CanonicalBasisValue::SignedInteger {
                    width: worth_foundational::CanonicalIntegerWidth::Bits64,
                    value: 4,
                }
    }));
    assert_eq!(
        locator_ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::Locator
    );
    assert!(locator_ready.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("source.aspect.aspect_key".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("task.count".into())
    }));
}

#[test]
fn canonicalization_basis_front_door_prepares_value_locator_surfaces() {
    let locator_ready = match canonicalization().basis().at(version()).from_value_locator(
        worth_foundational::AspectValueLocator::whole_aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::Authoritative,
            key("task.count"),
        )),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("value locator basis should be ready"),
    };

    assert_eq!(
        locator_ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::Locator
    );
    assert!(locator_ready.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("value.kind".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("whole_aspect".into())
    }));
    assert!(locator_ready.payload().entries().iter().any(|entry| {
        entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("value.whole_aspect.aspect_key".into())
            && entry.value() == &worth_foundational::CanonicalBasisValue::ExactText("task.count".into())
    }));
}

#[test]
fn canonicalization_digest_front_door_keeps_basis_authority_upstream_of_digest_output() {
    let ready_for_digest = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 9))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    };
    let digest_ready = match canonicalization().digest().for_sequence(
        ready_for_digest,
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest derivation should be ready"),
    };
    let digest = canonicalization().digest().derive(digest_ready);
    let left_for_comparison = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 9))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left comparison basis should be ready"),
    };
    let right_for_comparison = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 9))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right comparison basis should be ready"),
    };
    let comparison = match canonicalization()
        .compare()
        .left(left_for_comparison)
        .right(right_for_comparison)
        .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };

    assert_eq!(digest.metadata().algorithm().id().as_str(), "sha256");
    assert!(matches!(
        canonicalization().compare().evaluate(&comparison),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}

#[test]
fn canonicalization_front_door_exposes_unsupported_and_manifest_mismatch_inspection() {
    let left = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 3))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left basis should be ready"),
    };
    let right = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 3))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right basis should be ready"),
    };
    let unsupported_ready = match canonicalization()
        .compare()
        .left(left)
        .right(right)
        .under(worth_foundational::CanonicalEquivalenceBasis::ProjectionEquivalent)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("unsupported comparison should still prepare"),
    };
    let unsupported_outcome = canonicalization().compare().evaluate(&unsupported_ready);
    let unsupported = canonicalization()
        .compare()
        .unsupported_basis(&unsupported_outcome)
        .expect("unsupported mismatch basis should be visible");

    assert_eq!(
        unsupported.kind(),
        worth_foundational::CanonicalMismatchKind::UnsupportedComparison
    );
    assert!(canonicalization()
        .compare()
        .equivalent_basis(&unsupported_outcome)
        .is_none());
    assert!(canonicalization()
        .compare()
        .mismatch_basis(&unsupported_outcome)
        .is_none());

    let export_left_basis = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 5))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left export basis should be ready"),
    };
    let export_right_basis = match canonicalization()
        .basis()
        .at(version())
        .from_state(admitted_scalar_state("task.count", 10, 5))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right export basis should be ready"),
    };
    let export_left_bundle = match canonicalization()
        .basis()
        .at(version())
        .bundle([export_left_basis])
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left export bundle should be ready"),
    };
    let export_right_bundle = match canonicalization()
        .basis()
        .at(version())
        .bundle([export_right_basis])
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right export bundle should be ready"),
    };
    let left_export = match canonicalization()
        .export()
        .from_bundle(export_left_bundle)
        .named("front-door-left")
        .for_producer_shape(worth_foundational::CanonicalProducerShape::GoldenFixture)
        .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left export should be ready"),
    };
    let right_export = match canonicalization()
        .export()
        .from_bundle(export_right_bundle)
        .named("front-door-right")
        .for_producer_shape(worth_foundational::CanonicalProducerShape::SupportReplay)
        .under(worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis)
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right export should be ready"),
    };
    let export_outcome = canonicalization()
        .export()
        .compare(&left_export, &right_export);
    let manifest_mismatch = canonicalization()
        .export()
        .manifest_mismatch(&export_outcome)
        .expect("manifest mismatch should be visible");

    assert_eq!(
        manifest_mismatch.kind(),
        worth_foundational::CanonicalExportManifestMismatchKind::ProducerShapeMismatch
    );
    assert!(canonicalization()
        .export()
        .mismatch_basis(&export_outcome)
        .is_none());
}

#[test]
fn canonicalization_front_door_exposes_readiness_checklist_and_phase_gate_inspection() {
    let report = canonicalization().readiness().report();
    let gate = canonicalization()
        .readiness()
        .phase_gate(&report, worth_foundational::CanonicalMilestone2PhaseGate::DigestSlots)
        .expect("digest-slots phase gate should be visible");

    assert!(canonicalization().readiness().passes(&report));
    assert_eq!(gate.gate(), worth_foundational::CanonicalMilestone2PhaseGate::DigestSlots);
}
