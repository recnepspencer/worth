//! An inline page is historically retired only when the ordered history wrote
//! every image of it and retired every record on the last one, and only that
//! exact last image anchors the older ones.

use super::super::supersession::fixture::{admitted_images, image};
use super::*;
use crate::redo_replay::plan::supersession::observed_predecessors;
use crate::HistoricalRetiredTargetWitness;

const LATER: [u8; 32] = [0x53; 32];

/// A record of the fixture page.
fn placed(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

/// Three images of the fixture page, holding records 1, 1..=2 and 1..=3.
fn images() -> Vec<PhysicalRedoMemberInput> {
    vec![
        image(1, 1, 10, 1, 1, false).0,
        image(2, 2, 11, 2, 1, false).0,
        image(3, 3, 12, 3, 1, false).0,
    ]
}

/// The admitted images, then one member per retirement of page records.
fn page(
    images: Vec<PhysicalRedoMemberInput>,
    retirements: &[([u8; 32], &[u64])],
) -> AdmittedPhysicalRedoMembers {
    let mut admitted = admitted_images(images);
    let mut members = std::mem::take(&mut admitted.members).into_vec();
    members.extend(retirements.iter().map(|(operation, dropped)| {
        let dropped = dropped.iter().copied().map(placed).collect();
        member(
            *operation,
            projection(record(10), (5, 1), retirement(dropped)),
        )
    }));
    admitted.members = members.into_boxed_slice();
    admitted
}

/// Records 1 and 2 retired by `RETIRING`, then record 3 by `LATER`.
fn retired_page() -> AdmittedPhysicalRedoMembers {
    page(images(), &[(RETIRING, &[1, 2]), (LATER, &[3])])
}

/// One ordinary edge per admitted member, in member order.
fn edges(admitted: &AdmittedPhysicalRedoMembers) -> Vec<EdgeIdentity> {
    admitted
        .members
        .iter()
        .map(|member| EdgeIdentity {
            operation: member.operation,
            group: member.group,
            fate: member.fate,
            redo_sha256: member.canonical_redo_sha256,
            ordinary: true,
        })
        .collect()
}

fn image_target(admitted: &AdmittedPhysicalRedoMembers, member: usize) -> &PhysicalRedoTarget {
    &admitted.members[member].records[0].targets()[0]
}

/// The witness of the image this member wrote, under a root routing nothing.
fn witness(
    admitted: &AdmittedPhysicalRedoMembers,
    member: usize,
    edges: &[EdgeIdentity],
) -> Option<([u8; 32], [u8; 32])> {
    history(admitted, edges, &[]).retired_page(image_target(admitted, member))
}

fn retired_observation(
    target: &PhysicalRedoTarget,
    old_operation: [u8; 32],
) -> RecoveryPageObservation {
    RecoveryPageObservation::historical_retired_target(
        target,
        HistoricalRetiredTargetWitness {
            selected_root_identity: SELECTED,
            retiring_operation: LATER,
            old_operation,
            target: target.identity(),
            wal_target_digest: target.resulting_digest(),
            target_coordinate: RecordFrameCoordinate::new(
                target.artifact(),
                target.artifact_offset(),
                target.artifact_length(),
            )
            .unwrap(),
        },
    )
    .unwrap()
}

#[test]
fn a_page_is_retired_when_the_history_wrote_it_and_retired_every_record_on_it() {
    let admitted = retired_page();
    let ordered = edges(&admitted);
    assert_eq!(witness(&admitted, 2, &ordered), Some((SELECTED, LATER)));
    // The witness names the retirement ordered last, whichever record it drops.
    let swapped = [ordered[0], ordered[1], ordered[2], ordered[4], ordered[3]];
    assert_eq!(witness(&admitted, 2, &swapped), Some((SELECTED, RETIRING)));
}

#[test]
fn a_page_the_ordered_history_did_not_wholly_write_is_never_retired() {
    let admitted = retired_page();
    let ordered = edges(&admitted);
    // An older image is not the one the retirements emptied.
    assert_eq!(witness(&admitted, 1, &ordered), None);
    // An image above the history: no edge wrote the first one, or the last.
    assert_eq!(witness(&admitted, 2, &ordered[1..]), None);
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[1], ordered[3], ordered[4]]
        ),
        None
    );
    // Two edges that claim one image are ambiguous.
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[0], ordered[1], ordered[2], ordered[3], ordered[4]]
        ),
        None
    );
    // A page generation the WAL never held breaks the chain of images.
    let first = || image(1, 1, 10, 1, 1, false).0;
    let whole = page(
        vec![first(), image(2, 2, 11, 2, 1, false).0],
        &[(RETIRING, &[1, 2])],
    );
    assert_eq!(
        witness(&whole, 1, &edges(&whole)),
        Some((SELECTED, RETIRING))
    );
    let gapped = page(
        vec![first(), image(3, 2, 11, 2, 1, false).0],
        &[(RETIRING, &[1, 2])],
    );
    assert_eq!(witness(&gapped, 1, &edges(&gapped)), None);
}

#[test]
fn a_page_holding_a_record_the_history_did_not_retire_is_never_retired() {
    let admitted = retired_page();
    let ordered = edges(&admitted);
    // No admitted retirement: unrouted and already allocated is never enough.
    assert_eq!(witness(&admitted, 2, &ordered[..3]), None);
    // Retirements that cover only some records on the page.
    assert_eq!(witness(&admitted, 2, &ordered[..4]), None);
    let partial = page(images(), &[(RETIRING, &[1, 2])]);
    assert_eq!(witness(&partial, 2, &edges(&partial)), None);
    // A retirement ordered before the edge that wrote the image.
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[1], ordered[3], ordered[2], ordered[4]]
        ),
        None
    );
    // A released edge carrying the retiring identity is not an ordinary step.
    let released = EdgeIdentity {
        ordinary: false,
        ..ordered[4]
    };
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[1], ordered[2], ordered[3], released]
        ),
        None
    );
    // Duplicate retiring edges are ambiguous.
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[1], ordered[2], ordered[3], ordered[3], ordered[4]]
        ),
        None
    );
    // A damaged retiring edge binds no admitted member.
    let damaged = EdgeIdentity {
        redo_sha256: [0xee; 32],
        ..ordered[4]
    };
    assert_eq!(
        witness(
            &admitted,
            2,
            &[ordered[0], ordered[1], ordered[2], ordered[3], damaged]
        ),
        None
    );
    // A record the selected root still routes was never retired.
    assert_eq!(
        history(&admitted, &ordered, &[placed(3)]).retired_page(image_target(&admitted, 2)),
        None
    );
}

#[test]
fn the_entry_mints_a_witness_only_for_a_page_the_selected_root_no_longer_routes() {
    let admitted = retired_page();
    let ordered = edges(&admitted);
    let target = image_target(&admitted, 2);
    let witness = history(&admitted, &ordered, &[placed(9)])
        .admit(target)
        .expect("the history wrote the page and retired every record on it");
    assert_eq!(
        RecoveryPageObservation::historical_retired_target(target, witness),
        Some(retired_observation(target, admitted.members[2].operation))
    );
    // The selected root still routes one record of the page, whichever it is.
    for routed in 1..=3 {
        assert!(history(&admitted, &ordered, &[placed(routed)])
            .admit(target)
            .is_none());
    }
    // A history that is not anchored on the selection mints no witness.
    let unanchored =
        HistoricalRetirements::index(&admitted.members, ordered.clone(), None, std::iter::empty());
    assert!(unanchored.admit(target).is_none());
    // An older image of the page is never the retired one.
    assert!(history(&admitted, &ordered, &[])
        .admit(image_target(&admitted, 1))
        .is_none());
}

#[test]
fn an_image_is_emptied_only_by_retirements_of_every_record_it_holds() {
    let admitted = retired_page();
    let ordered = edges(&admitted);
    let history = history(&admitted, &ordered, &[]);
    let records = |ordinals: &[u64]| ordinals.iter().copied().map(placed).collect::<Vec<_>>();
    let emptied =
        |ordinals: &[u64], written| history.emptied_at(records(ordinals).into_iter(), written);
    assert_eq!(emptied(&[1, 2, 3], 2), Some((4, SELECTED)));
    // An image that holds no record proves no retirement.
    assert_eq!(emptied(&[], 2), None);
    // The edge that wrote the image may itself retire records on it.
    assert_eq!(emptied(&[1, 2], 3), Some((3, SELECTED)));
    // A retirement ordered before the write is denied, though the producer
    // never appends to a page after retiring one of its records.
    assert_eq!(emptied(&[1, 2], 4), None);
    assert_eq!(emptied(&[1, 2, 3], 4), None);
    // A record no ordered edge retired.
    assert_eq!(emptied(&[1, 4], 2), None);
}

#[test]
fn the_retired_last_image_anchors_every_older_image_of_its_page() {
    let admitted = admitted_images(images());
    let observed = retired_observation(image_target(&admitted, 2), admitted.members[2].operation);
    assert_eq!(
        observed_predecessors(&admitted.members, &[observed]).unwrap(),
        BTreeSet::from([
            (10, image_target(&admitted, 0).identity()),
            (11, image_target(&admitted, 1).identity()),
        ])
    );
    let plan = admitted.plan(vec![observed]).unwrap();
    assert_eq!(plan.decisions().len(), 3);
    assert!(plan.decisions().iter().all(|decision| {
        decision.kind() == PhysicalRedoDecisionKind::SkipHistoricallyRetiredTarget
    }));
}

#[test]
fn a_witness_that_does_not_name_the_exact_last_image_anchors_nothing() {
    // Another operation named as the writer of the image.
    let admitted = admitted_images(images());
    let forged = retired_observation(image_target(&admitted, 2), [0xee; 32]);
    assert!(observed_predecessors(&admitted.members, &[forged])
        .unwrap()
        .is_empty());
    assert_eq!(
        admitted.plan(vec![forged]).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch)
    );
    // Another image at the same page generation and coordinate.
    let changed = admitted_images(vec![image(3, 3, 12, 3, 1, true).0]);
    let forged = retired_observation(image_target(&changed, 0), changed.members[0].operation);
    let admitted = admitted_images(images());
    assert!(observed_predecessors(&admitted.members, &[forged])
        .unwrap()
        .is_empty());
    assert_eq!(
        admitted.plan(vec![forged]).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch)
    );
    // An older image named as retired: the later image is never skipped.
    let admitted = admitted_images(images());
    let older = retired_observation(image_target(&admitted, 1), admitted.members[1].operation);
    assert_eq!(
        observed_predecessors(&admitted.members, &[older]).unwrap(),
        BTreeSet::from([(10, image_target(&admitted, 0).identity())])
    );
    assert_eq!(
        admitted.plan(vec![older]).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch)
    );
    // A page generation the WAL never held breaks the chain under the witness.
    let gapped = admitted_images(vec![
        image(1, 1, 10, 1, 1, false).0,
        image(3, 2, 11, 2, 1, false).0,
    ]);
    let observed = retired_observation(image_target(&gapped, 1), gapped.members[1].operation);
    assert_eq!(
        observed_predecessors(&gapped.members, &[observed]).err(),
        Some(PhysicalRedoPlanningDenial::GenerationMismatch)
    );
}
