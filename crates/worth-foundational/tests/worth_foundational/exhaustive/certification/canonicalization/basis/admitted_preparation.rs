#[allow(unused_imports)]
pub(crate) use crate::fast::certification::canonicalization::basis::admitted_preparation::*;

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
            let expected: Vec<_> = (0..count).map(entry).collect();
            assert_eq!(admitted.payload().entries(), expected.as_slice());
            assert!(charged.0 >= admitted.payload().cost().ordering_comparisons() as usize);
            assert!(charged.1 >= version().as_str().len());
        }
    }
}
