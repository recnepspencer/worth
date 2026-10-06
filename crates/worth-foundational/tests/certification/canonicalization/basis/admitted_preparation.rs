use worth_foundational::facade::{
    prepare_canonical_basis_sequence, prepare_owned_canonical_basis_sequence_admitted, AspectKey,
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisPreparationStop,
    CanonicalBasisValue, CanonicalFieldPath, CanonicalizationRuleVersion, FieldKey,
};

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("admitted.preparation.v1").unwrap()
}

fn entry(ordinal: usize) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named(format!("field.{ordinal:04}").into()),
        CanonicalBasisEntryKind::Field,
        CanonicalBasisValue::ExactText(format!("value.{ordinal}").into()),
    )
}

#[test]
fn admitted_preparation_preserves_entry_order_and_actual_comparison_evidence() {
    // Cross insertion, small-sort, eager merging and quicksort thresholds.
    for count in [1, 20, 21, 32, 65, 257] {
        for order in 0..3 {
            let mut entries: Vec<_> = (0..count).map(entry).collect();
            if order == 1 {
                entries.reverse();
            } else if order == 2 {
                // A bijection with interleaved runs, deterministic without RNG.
                entries.rotate_left(count / 3);
                entries[..count / 2].reverse();
            }
            let ordinary = prepare_canonical_basis_sequence(
                version(),
                CanonicalBasisDomain::Value,
                entries.clone(),
            )
            .into_result()
            .unwrap();
            let mut charged = (0usize, 0usize);
            let admitted = prepare_owned_canonical_basis_sequence_admitted(
                version(),
                CanonicalBasisDomain::Value,
                entries,
                |work, bytes| {
                    charged.0 = charged.0.checked_add(work).unwrap();
                    charged.1 = charged.1.checked_add(bytes).unwrap();
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
            assert_eq!(admitted.payload(), ordinary.payload());
            assert_eq!(admitted.payload().entries()[0], entry(0));
            assert_eq!(admitted.payload().entries()[count - 1], entry(count - 1));
            assert!(charged.0 >= admitted.payload().cost().ordering_comparisons() as usize);
            assert!(charged.1 >= version().as_str().len());
        }
    }
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
        assert_eq!(
            result.unwrap_err(),
            CanonicalBasisPreparationStop::Resource("original refusal")
        );
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
