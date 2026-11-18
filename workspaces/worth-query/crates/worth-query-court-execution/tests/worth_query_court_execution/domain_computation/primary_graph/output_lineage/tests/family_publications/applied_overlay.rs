//! Differential native index selection: an applied replacement cannot hide another partition.
use super::*;

#[test]
fn applied_overlay_preserves_unaffected_heads_and_partition_scoped_absence() {
    let mut court = Court::default();
    court.publish::<Initial>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    let unaffected = court.publish::<Initial>(
        3,
        Some(2),
        3,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    let execution = worth_execution::ExecutionRequest::serial(&court.request);
    let select = |replaced: &[(Option<TypeId>, [u8; 32])]| {
        court
            .lineage
            .resolve_current_family_overlaid_for_test(
                &execution,
                *court.source.runtime_authority_for_test(),
                court.source.schema_for_test(),
                court.source.scope,
                "family",
                *court.coordinate.occurrence_for_test(),
                *court.coordinate.generation_for_test(),
                replaced,
            )
            .unwrap()
            .candidates_for_test()
            .iter()
            .map(|candidate| Arc::clone(candidate.correspondence_for_test()))
            .collect::<Vec<_>>()
    };
    let before = court.resolve();
    let unchanged = select(&[]);
    assert_eq!(before.candidates_for_test().len(), 2);
    assert_eq!(unchanged.len(), before.candidates_for_test().len());
    for (old, new) in before.candidates_for_test().iter().zip(&unchanged) {
        assert!(Arc::ptr_eq(old.correspondence_for_test(), new));
    }
    // Both replacement and proven absence of A retain the actual B head.
    for replaced in [
        vec![(Some(TypeId::of_for_test::<Initial>()), [1; 32])],
        vec![(None, [1; 32])],
    ] {
        let selected = select(&replaced);
        assert_eq!(selected.len(), 1);
        assert!(Arc::ptr_eq(&selected[0], &unaffected));
    }
    assert_eq!(
        select(&[(Some(TypeId::of_for_test::<Preserve>()), [1; 32])]).len(),
        2
    );
}
