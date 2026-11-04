//! Descriptive index selection with inherited settlements; no current seal.
use crate::domain_computation::primary_graph::output_lineage::tests::family_publications::*;

#[test]
fn actual_publication_generation_supersedes_inherited_settlement_generation() {
    let mut court = Court::new();
    court.publish::<Initial>(
        5,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    court.publish::<Preserve>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        6,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    court.inherit_settlement::<Preserve, Preserve>(6, 2);
    let selected = court.resolve();
    assert!(!(*selected.ambiguous_publication_for_test()));
    assert_eq!((*selected.candidates_for_test()).len(), 1);
    assert_eq!(
        (*selected.candidates_for_test())[0].correspondence.binding_type(),
        Some(TypeId::of_for_test::<Preserve>())
    );
    assert_eq!((*selected.candidates_for_test())[0].settlement_identity.address().1, 2);
}

#[test]
fn equal_publication_generation_is_ambiguous_despite_inherited_settlement() {
    let mut court = Court::new();
    court.publish::<Initial>(
        5,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        5,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    court.inherit_settlement::<Preserve, Preserve>(5, 2);
    for _ in 0..2 {
        let selected = court.resolve();
        assert!((*selected.ambiguous_publication_for_test()));
        assert!((*selected.candidates_for_test()).is_empty());
        (*court
            .lineage
            .output_families_for_test())
            .get_mut("family")
            .unwrap()
            .reverse();
    }
}
