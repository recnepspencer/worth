//! A home's four anchors: the canonical commits, the branch transitions, the
//! home descriptor and the checkpoint manifest share the root's anchor table,
//! never an anchor, and the table refuses a fifth before any effect.

use worth_store::physical_runtime::{
    AnchoredRecordPublicationFailure, OrderedJournalPublicationFailure,
};

use super::canonical_journal_oracle::{placement, serve};
use super::two_journals::{assert_holds, publish, Journal};
use super::*;
use crate::anchored_records::{
    current, name, publish as publish_anchored, value, CHECKPOINT, HOME,
};

#[test]
fn both_journals_and_both_anchored_records_fill_the_anchor_table() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("four-anchors")
        .expect("anchor table store");
    let serving = world.serving();
    let (home, checkpoint) = (name(HOME), name(CHECKPOINT));
    publish(serving, world.placement(), Journal::Commits, 1).expect("commits anchor");
    publish_anchored(serving, world.placement(), home, 0, value(1, 200)).expect("home anchor");
    publish(serving, world.placement(), Journal::Transitions, 1).expect("transitions anchor");
    publish_anchored(serving, world.placement(), checkpoint, 0, value(2, 900))
        .expect("checkpoint anchor");

    let quiet = serving.physical_mutation_observation();
    assert!(matches!(
        publish_anchored(
            serving,
            world.placement(),
            name("anchor.five"),
            0,
            value(5, 64)
        ),
        Err(AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "the refusal wrote"
    );

    // Every anchor keeps advancing within its own slot.
    publish(serving, world.placement(), Journal::Commits, 2).expect("second commit");
    publish(serving, world.placement(), Journal::Transitions, 2).expect("second transition");
    publish_anchored(serving, world.placement(), checkpoint, 1, value(3, 900))
        .expect("second checkpoint");
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "four anchors");
    assert_holds(&recovered, Journal::Commits, 2);
    assert_holds(&recovered, Journal::Transitions, 2);
    assert_eq!(current(&recovered, home), Some((1, value(1, 200))));
    assert_eq!(current(&recovered, checkpoint), Some((2, value(3, 900))));
    assert!(matches!(
        publish_anchored(
            &recovered,
            placement(),
            name("anchor.five"),
            0,
            value(5, 64)
        ),
        Err(AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    let _ = recovered.close();
}

#[test]
fn a_journal_first_entry_refuses_a_full_anchor_table() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("journal-table-full")
        .expect("anchor table store");
    let serving = world.serving();
    publish(serving, world.placement(), Journal::Commits, 1).expect("commits anchor");
    for (index, text) in ["anchor.one", "anchor.two", "anchor.three"]
        .into_iter()
        .enumerate()
    {
        publish_anchored(
            serving,
            world.placement(),
            name(text),
            0,
            value(index as u8, 64),
        )
        .expect("an anchor slot is free");
    }
    let quiet = serving.physical_mutation_observation();
    assert!(matches!(
        publish(serving, world.placement(), Journal::Transitions, 1),
        Err(OrderedJournalPublicationFailure::AnchorTableFull)
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "the refusal wrote"
    );
    assert_holds(serving, Journal::Transitions, 0);
    // A journal that already holds its anchor keeps publishing.
    publish(serving, world.placement(), Journal::Commits, 2).expect("second commit");
    assert_holds(serving, Journal::Commits, 2);
}
