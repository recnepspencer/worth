//! Ordered publication heads are selected without caller work estimates.
use super::*;

#[test]
fn family_publication_selection_preserves_ordered_heads_without_billing() {
    for population in [1_u8, 16, 128] {
        let mut court = Court::new();
        let mut expected = Vec::new();
        for partition in 0..population {
            court.publish::<Initial>(
                1,
                Some(partition),
                u64::from(partition) + 2,
                WorthQueryApplicationOutputPosture::Create,
                true,
            );
            expected.push(court.publish::<Initial>(
                2,
                Some(partition),
                u64::from(partition) + 2,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            ));
        }
        let selected = court.resolve();

        assert_eq!(selected.candidates.len(), usize::from(population));
        for (candidate, expected) in selected.candidates.iter().zip(&expected) {
            assert!(Arc::ptr_eq(&candidate.correspondence, expected));
        }
        assert!(!selected.ambiguous_publication);
    }
}
