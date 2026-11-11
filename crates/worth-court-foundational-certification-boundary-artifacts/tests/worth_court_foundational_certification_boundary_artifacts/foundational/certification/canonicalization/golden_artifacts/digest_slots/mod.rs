// V2 golden: independently SHA-256 hashed the explicit token fixture.
// 0cd5a1f55058b76c5e93caf9fa8b6ff87543f74e02dc35af2d0353346f7fd686
use {
    worth_foundational::admit_canonical_sequence_digest_derivation,
    worth_foundational::derive_canonical_digest,
    worth_foundational::prepare_canonical_basis_sequence,
};

#[test]
fn admitted_sequence_preserves_golden_bytes_and_original_resource_refusal() {
    use {
        worth_foundational::canonicalization, worth_foundational::CanonicalDigestAdmissionStop,
        worth_foundational::CanonicalDigestWorkBudget,
    };
    let make_basis = || {
        prepare_canonical_basis_sequence(
            worth_foundational::CanonicalizationRuleVersion::new("m2.golden.digest-slot")
                .expect("valid version"),
            worth_foundational::CanonicalBasisDomain::Value,
            [worth_foundational::CanonicalBasisEntry::new(
                worth_foundational::CanonicalBasisDomain::Value,
                worth_foundational::CanonicalBasisLocus::Named("digest-golden".into()),
                worth_foundational::CanonicalBasisEntryKind::Value,
                worth_foundational::CanonicalBasisValue::SignedInteger {
                    width: worth_foundational::CanonicalIntegerWidth::Bits64,
                    value: 42,
                },
            )],
        )
        .into_result()
        .expect("real canonical basis")
    };
    let mut work = 0usize;
    let mut bytes = 0usize;
    let mut admit = |next_work: usize, next_bytes: usize| -> Result<(), &'static str> {
        work = work.checked_add(next_work).expect("fixture Work fits");
        bytes = bytes.checked_add(next_bytes).expect("fixture bytes fit");
        Ok(())
    };
    let ready = canonicalization()
        .digest()
        .for_sequence_with_admission(
            make_basis(),
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            CanonicalDigestWorkBudget::standard(),
            &mut admit,
        )
        .expect("funded canonical preparation");
    let digest = canonicalization()
        .digest()
        .derive_with_admission(ready, &mut admit)
        .expect("funded hash and metadata");
    assert_eq!(
        digest.value().bytes(),
        &[
            12, 213, 161, 245, 80, 88, 183, 108, 94, 147, 202, 249, 250, 139, 111, 248, 117, 67,
            247, 78, 2, 220, 53, 175, 45, 3, 83, 52, 111, 127, 214, 134,
        ]
    );
    assert_eq!(
        digest.metadata().input_id().as_str(),
        "sequence:value:m2.golden.digest-slot:1"
    );
    assert!(work > digest.metadata().work().canonical_encoded_bytes());
    assert!(
        bytes
            >= digest
                .metadata()
                .work()
                .canonical_material_allocation_bytes()
    );
    let mut calls = 0;
    let result = canonicalization().digest().for_sequence_with_admission(
        make_basis(),
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        CanonicalDigestWorkBudget::standard(),
        &mut |_, _| {
            calls += 1;
            Err("original refusal")
        },
    );
    assert!(matches!(
        result,
        Err(CanonicalDigestAdmissionStop::Resource("original refusal"))
    ));
    assert_eq!(calls, 1, "refusal stops before any subsequent construction");
}

#[test]
fn sha256_digest_slot_has_stable_derived_value() {
    let version = worth_foundational::CanonicalizationRuleVersion::new("m2.golden.digest-slot")
        .expect("valid version");
    let sequence = match prepare_canonical_basis_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Value,
        [worth_foundational::CanonicalBasisEntry::new(
            worth_foundational::CanonicalBasisDomain::Value,
            worth_foundational::CanonicalBasisLocus::Named("digest-golden".into()),
            worth_foundational::CanonicalBasisEntryKind::Value,
            worth_foundational::CanonicalBasisValue::SignedInteger {
                width: worth_foundational::CanonicalIntegerWidth::Bits64,
                value: 42,
            },
        )],
    ) {
        worth_proof::TransitionOutcome::Success(sequence) => sequence,
        _ => panic!("golden digest sequence should be ready"),
    };
    let ready = match admit_canonical_sequence_digest_derivation(
        sequence,
        worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            worth_foundational::CanonicalDigestAlgorithmId::sha256(),
            worth_foundational::CanonicalBasisDomain::Value,
            version,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("golden digest derivation should be admitted"),
    };

    let digest = derive_canonical_digest(ready);

    assert_eq!(
        digest.value().bytes(),
        &[
            12, 213, 161, 245, 80, 88, 183, 108, 94, 147, 202, 249, 250, 139, 111, 248, 117, 67,
            247, 78, 2, 220, 53, 175, 45, 3, 83, 52, 111, 127, 214, 134,
        ]
    );
}
