pub(crate) use worth_foundational::facade::{
    prepare_canonical_basis_sequence, prepare_owned_canonical_basis_sequence_admitted, AspectKey,
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisPreparationStop,
    CanonicalBasisValue, CanonicalFieldPath, CanonicalizationRuleVersion, FieldKey,
};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("admitted.preparation.v1").unwrap()
}

pub(crate) fn entry(ordinal: usize) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named(format!("field.{ordinal:04}").into()),
        CanonicalBasisEntryKind::Field,
        CanonicalBasisValue::ExactText(format!("value.{ordinal}").into()),
    )
}

#[test]
fn refused_measurement_or_sorting_never_emits_ready_and_preserves_refusal() {
    for refused_call in [1, 4] {
        let mut calls = 0;
        let result = prepare_owned_canonical_basis_sequence_admitted(
            version(),
            CanonicalBasisDomain::Value,
            vec![entry(1), entry(0)],
            |_, _| {
                calls += 1;
                if calls == refused_call {
                    Err("original refusal")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            result.unwrap_err(),
            CanonicalBasisPreparationStop::Resource {
                cause: "original refusal",
                ..
            }
        ));
        assert_eq!(calls, refused_call);
    }
}

#[test]
fn domain_rejection_precedes_measurement_and_sort_storage() {
    let mut calls = Vec::new();
    let result = prepare_owned_canonical_basis_sequence_admitted(
        version(),
        CanonicalBasisDomain::Identity,
        vec![entry(0)],
        |work, bytes| {
            calls.push((work, bytes));
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result.unwrap_err(),
        CanonicalBasisPreparationStop::Construction(
            CanonicalBasisConstructionDenial::DomainMismatch {
                expected: CanonicalBasisDomain::Identity,
                actual: CanonicalBasisDomain::Value,
            },
        )
    );
    assert_eq!(calls, [(4, 0)]);
}

#[test]
fn duplicate_field_path_denial_retains_its_exact_owned_locus() {
    let locus = CanonicalBasisLocus::AspectField {
        aspect: AspectKey::new("bank.Account").unwrap(),
        path: CanonicalFieldPath::new([
            FieldKey::new("balance").unwrap(),
            FieldKey::new("currency").unwrap(),
        ])
        .unwrap(),
    };
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        locus.clone(),
        CanonicalBasisEntryKind::Field,
        CanonicalBasisValue::Bool(true),
    );
    let mut allocation = 0;
    let result = prepare_owned_canonical_basis_sequence_admitted(
        version(),
        CanonicalBasisDomain::Value,
        vec![entry.clone(), entry],
        |_, bytes| {
            allocation += bytes;
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result.unwrap_err(),
        CanonicalBasisPreparationStop::Construction(
            CanonicalBasisConstructionDenial::DuplicateEntry {
                domain: CanonicalBasisDomain::Value,
                locus,
                kind: CanonicalBasisEntryKind::Field,
            },
        )
    );
    assert!(allocation >= "bank.Accountbalancecurrency".len() + 2 * size_of::<FieldKey>());
}
