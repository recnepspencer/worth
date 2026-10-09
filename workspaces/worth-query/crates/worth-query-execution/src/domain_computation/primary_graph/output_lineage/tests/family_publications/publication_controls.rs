//! Descriptive publication ordering and partition separation.
use super::*;

#[test]
fn newer_preserve_publication_supersedes_initial_even_without_initial_facts() {
    for (partition, initial_facts) in [(Some(1), true), (Some(1), false), (None, true)] {
        let mut court = Court::new();
        court.publish::<Initial>(
            2,
            partition,
            2,
            WorthQueryApplicationOutputPosture::Create,
            initial_facts,
        );
        let preserve = court.publish::<Preserve>(
            3,
            partition,
            2,
            WorthQueryApplicationOutputPosture::Preserve,
            true,
        );
        let selected = court.resolve();
        assert_eq!(
            selected.candidates.len(),
            1,
            "the obsolete binding must not supply a retry packet"
        );
        assert!(Arc::ptr_eq(
            &selected.candidates[0].correspondence,
            &preserve
        ));
        assert!(!selected.ambiguous_publication);
    }
}

#[test]
fn retirement_and_descriptive_heads_cannot_resurrect_the_initial_binding() {
    for (posture, facts) in [
        (WorthQueryApplicationOutputPosture::Retire, false),
        (WorthQueryApplicationOutputPosture::Retire, true),
        (WorthQueryApplicationOutputPosture::Preserve, false),
    ] {
        let mut court = Court::new();
        court.publish::<Initial>(
            2,
            Some(1),
            2,
            WorthQueryApplicationOutputPosture::Create,
            true,
        );
        court.publish::<Preserve>(3, Some(1), 2, posture, facts);
        let selected = court.resolve();
        assert!(!selected.ambiguous_publication);
        assert_eq!(selected.candidates.len(), usize::from(facts));
        if facts {
            assert_eq!(
                selected.candidates[0]
                    .correspondence
                    .active_entity_for_role("output"),
                None
            );
            assert_eq!(selected.candidates[0].settlement_identity.address().1, 3);
        }
    }
}

#[test]
fn distinct_partitions_entities_and_roles_remain_distinct_family_members() {
    for (partition, output, other_role) in [
        (Some(2), 2, false),
        (None, 2, false),
        (Some(1), 3, false),
        (Some(1), 2, true),
    ] {
        let mut court = Court::new();
        court.publish::<Initial>(
            2,
            Some(1),
            2,
            WorthQueryApplicationOutputPosture::Create,
            true,
        );
        if other_role {
            court
                .lineage
                .output_families
                .get_mut("family")
                .unwrap()
                .push((TypeId::of::<OtherRole>(), "other".to_owned()));
            court.publish::<OtherRole>(
                3,
                partition,
                output,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            );
        } else {
            court.publish::<Preserve>(
                3,
                partition,
                output,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            );
        }
        let selected = court.resolve();
        assert_eq!(selected.candidates.len(), 2);
        assert!(!selected.ambiguous_publication);
    }
}

#[test]
fn equal_publication_positions_refuse_selection_in_both_binding_orders() {
    let mut court = Court::new();
    court.publish::<Initial>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    for _ in 0..2 {
        let selected = court.resolve();
        assert!(selected.ambiguous_publication);
        assert!(
            selected.candidates.is_empty(),
            "no iteration-order retry packet"
        );
        court
            .lineage
            .output_families
            .get_mut("family")
            .unwrap()
            .reverse();
    }
}

#[test]
fn selected_descendant_supersedes_ancestor_even_with_lower_generation() {
    let mut court = Court::new();
    court.publish::<Initial>(
        9,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    let ancestor = court.coordinate;
    // Local index coordinates are descriptive; the real native branch identities
    // above supply the ancestry distinction without granting currentness.
    court.lineage.origins.insert(court.child, ancestor);
    court.coordinate = ProductCoordinate {
        occurrence: court.child,
        generation: 2,
    };
    court.publish::<Preserve>(
        1,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    let selected = court.resolve();
    assert_eq!(selected.candidates.len(), 1);
    assert_eq!(
        selected.candidates[0].settlement_identity.address().0,
        court.child
    );
    assert_eq!(selected.candidates[0].settlement_identity.address().1, 1);
    assert!(!selected.ambiguous_publication);
}
