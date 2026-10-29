#[allow(unused_imports)]
pub(crate) use {worth_foundational::facade::CanonicalBasisDomain, crate::certification::ordinary_context::canonicalization::basis::admitted_preparation::entry, worth_foundational::facade::prepare_canonical_basis_sequence, worth_foundational::facade::prepare_owned_canonical_basis_sequence_admitted, crate::certification::ordinary_context::canonicalization::basis::admitted_preparation::version};

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
                worth_foundational::facade::CanonicalBasisDomain::Value,
                entries.clone(),
            )
            .into_result()
            .unwrap();
            let mut charged = (0usize, 0usize);
            let admitted = prepare_owned_canonical_basis_sequence_admitted(
                version(),
                worth_foundational::facade::CanonicalBasisDomain::Value,
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
