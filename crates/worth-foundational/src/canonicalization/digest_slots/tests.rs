use super::*;
use crate::canonicalization::*;
use worth_proof::TransitionOutcome;

fn sequence(order: &[usize]) -> CanonicalBasisReadyArtifact {
    let entries = order.iter().map(|index| {
        CanonicalBasisEntry::new(
            CanonicalBasisDomain::Value,
            CanonicalBasisLocus::Named(format!("field-{index}").into()),
            CanonicalBasisEntryKind::Value,
            CanonicalBasisValue::ExactText(format!("value-{index}").into()),
        )
    });
    match prepare_canonical_basis_sequence(version(), CanonicalBasisDomain::Value, entries) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("valid basis: {other:?}"),
    }
}

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("digest-court").unwrap()
}

fn digest(order: &[usize]) -> CanonicalDerivedDigest {
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version(),
    );
    match admit_canonical_sequence_digest_derivation(sequence(order), slot) {
        TransitionOutcome::Success(ready) => derive_canonical_digest(ready),
        other => panic!("valid digest: {other:?}"),
    }
}

#[test]
fn every_permutation_and_repreparation_preserves_digest_identity() {
    fn permutations(order: &mut [usize], offset: usize, expected: &CanonicalDerivedDigest) {
        if offset == order.len() {
            let actual = digest(order);
            assert_eq!(actual.value(), expected.value(), "order {order:?}");
            assert_eq!(actual.metadata().input_id(), expected.metadata().input_id());
        } else {
            for index in offset..order.len() {
                order.swap(offset, index);
                permutations(order, offset + 1, expected);
                order.swap(offset, index);
            }
        }
    }
    let mut order = [0, 1, 2, 3, 4, 5];
    let expected = digest(&order);
    permutations(&mut order, 0, &expected);
    assert_ne!(digest(&[0, 1, 2, 3, 4, 6]).value(), expected.value());
}

#[test]
fn legacy_digest_policy_is_refused_before_derivation() {
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::new("sha256").unwrap(),
        CanonicalBasisDomain::Value,
        version(),
    );
    assert!(matches!(
        admit_canonical_sequence_digest_derivation(sequence(&[0]), slot),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::UnsupportedAlgorithm)
    ));
}

#[test]
fn streaming_matches_buffered_material_for_each_input_shape() {
    fn check(ready: CanonicalDigestDerivationReadyArtifact) {
        let input = ready.payload();
        let material = super::material::canonical_digest_material(
            input.algorithm(),
            input.evidence(),
            None,
            None,
        )
        .unwrap();
        let encoded = material.encoded_bytes();
        let expected = super::material::sha256_digest(&material.into_bytes());
        let actual = derive_canonical_digest(ready);
        assert_eq!(actual.value().bytes(), &expected);
        assert_eq!(actual.metadata().work().canonical_encoded_bytes(), encoded);
        assert_eq!(
            actual
                .metadata()
                .work()
                .canonical_material_allocation_bytes(),
            0
        );
    }
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version(),
    );
    let TransitionOutcome::Success(ready) =
        admit_canonical_sequence_digest_derivation(sequence(&[2, 0, 1]), slot)
    else {
        panic!("sequence digest")
    };
    check(ready);
    let bundle = || {
        let TransitionOutcome::Success(bundle) =
            prepare_canonical_basis_bundle(version(), [sequence(&[2, 0, 1])])
        else {
            panic!("bundle")
        };
        bundle
    };
    let slot = CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
        CanonicalDigestAlgorithmId::sha256(),
        version(),
    );
    let TransitionOutcome::Success(ready) =
        admit_canonical_bundle_digest_derivation(bundle(), slot)
    else {
        panic!("bundle digest")
    };
    check(ready);
    let TransitionOutcome::Success(export) = prepare_canonical_export_bundle(
        "streaming-court",
        CanonicalProducerShape::GoldenFixture,
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        bundle(),
    );
    let slot = CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
        CanonicalDigestAlgorithmId::sha256(),
        version(),
    );
    let TransitionOutcome::Success(ready) = admit_canonical_export_digest_derivation(export, slot)
    else {
        panic!("export digest")
    };
    check(ready);
}

#[test]
fn bundle_admission_moves_the_admitted_entry_allocation() {
    let basis = sequence(&[0, 1, 2]);
    let entries = basis.payload().entries().as_ptr();
    let TransitionOutcome::Success(bundle) = prepare_canonical_basis_bundle(version(), [basis])
    else {
        panic!("bundle")
    };
    let slot = CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
        CanonicalDigestAlgorithmId::sha256(),
        version(),
    );
    let TransitionOutcome::Success(ready) = admit_canonical_bundle_digest_derivation(bundle, slot)
    else {
        panic!("bundle digest")
    };
    let CanonicalDigestInputEvidence::DomainBundle(bundle) = ready.payload().evidence() else {
        panic!("bundle evidence")
    };
    assert_eq!(bundle.sequences()[0].entries().as_ptr(), entries);
}

#[test]
fn entry_and_byte_limits_return_typed_denials_without_digest_output() {
    let slot = || {
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Value,
            version(),
        )
    };
    let budget = CanonicalDigestWorkBudget::standard();
    let order = (0..=budget.maximum_entry_count() as usize).collect::<Vec<_>>();
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(
            sequence(&order),
            slot(),
            CanonicalDigestWorkBudget::standard()
        ),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::EntryLimitExceeded {
            maximum,
            actual
        }) if maximum == budget.maximum_entry_count() && actual > maximum
    ));
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(
            sequence(&[0]),
            slot(),
            CanonicalDigestWorkBudget::new(10, 1).unwrap()
        ),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            maximum: 1,
            ..
        })
    ));
}

#[test]
fn streaming_storage_admission_stays_flat_as_material_grows() {
    fn storage(entries: usize) -> usize {
        let ready = sequence(&(0..entries).collect::<Vec<_>>());
        let mut storage = 0;
        let mut admit = |_: usize, bytes: usize| -> Result<(), ()> {
            storage += bytes;
            Ok(())
        };
        let digest_ready = CanonicalDigestFrontDoor
            .for_sequence_with_admission(
                ready,
                CanonicalDigestAlgorithmId::sha256(),
                CanonicalDigestWorkBudget::standard(),
                &mut admit,
            )
            .unwrap();
        derive_canonical_digest_with_admission(digest_ready, &mut admit).unwrap();
        storage
    }
    let baseline = storage(128);
    for actual in [storage(256), storage(512)] {
        assert!(
            (actual as f64) / (baseline as f64) < 1.5,
            "material buffer grew: {baseline} -> {actual}"
        );
    }
}
