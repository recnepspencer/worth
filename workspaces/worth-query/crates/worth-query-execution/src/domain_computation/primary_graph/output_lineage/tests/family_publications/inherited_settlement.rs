//! Descriptive index selection with inherited settlements; no current seal.
use super::*;

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
    assert!(!selected.ambiguous_publication);
    assert_eq!(selected.candidates.len(), 1);
    assert_eq!(
        selected.candidates[0].correspondence.binding_type(),
        Some(TypeId::of::<Preserve>())
    );
    assert_eq!(selected.candidates[0].settlement_identity.address().1, 2);
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
        assert!(selected.ambiguous_publication);
        assert!(selected.candidates.is_empty());
        court
            .lineage
            .output_families
            .get_mut("family")
            .unwrap()
            .reverse();
    }
}
