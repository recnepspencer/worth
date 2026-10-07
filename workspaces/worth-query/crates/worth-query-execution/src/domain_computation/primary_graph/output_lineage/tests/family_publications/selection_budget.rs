//! Exact admitted selection work for descriptive partition heads.
use super::*;

#[test]
fn family_publication_selection_charges_bounded_work_for_offered_heads() {
    for (population, expected_work) in [(1_u8, 15), (16, 584), (128, 8_728)] {
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
        assert!(
            court.resolve_budgeted(expected_work - 1).is_err(),
            "selection comparisons must consume the work budget"
        );
        let selected = court
            .resolve_budgeted(expected_work)
            .expect("the declared head-selection budget is sufficient");
        assert_eq!(selected.selection_work, expected_work);
        assert_eq!(selected.candidates.len(), usize::from(population));
        for (candidate, expected) in selected.candidates.iter().zip(&expected) {
            assert!(Arc::ptr_eq(&candidate.correspondence, expected));
        }
        assert!(!selected.ambiguous_publication);
    }
}
