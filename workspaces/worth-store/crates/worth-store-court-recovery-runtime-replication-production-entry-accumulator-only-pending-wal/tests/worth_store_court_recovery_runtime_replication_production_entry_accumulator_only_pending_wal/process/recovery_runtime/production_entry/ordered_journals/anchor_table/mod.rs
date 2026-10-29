//! A home's four anchors: the canonical commits, the branch transitions, the
//! home descriptor and the checkpoint manifest share the root's 32-slot table.
//! Twenty-eight other records fill it; the next refuses before any effect.

use {worth_store::physical_runtime::AnchoredRecordPublicationFailure, worth_store::physical_runtime::OrderedJournalPublicationFailure};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
use {crate::production_entry::ordered_journals::two_journals::assert_holds, crate::production_entry::ordered_journals::two_journals::publish, crate::production_entry::ordered_journals::two_journals::Journal};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};
use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::current, worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish as publish_anchored, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT, worth_store_testkit_recovery_runtime::production_entry::anchored_records::HOME};

#[test]
fn both_journals_and_both_anchored_records_fill_the_anchor_table() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("mixed-32-anchors")
        .expect("anchor table store");
    let serving = world.serving();
    let (home, checkpoint) = (name(HOME), name(CHECKPOINT));
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Commits, 1).expect("commits anchor");
    publish_anchored(serving, world.placement(), home, 0, value(1, 200)).expect("home anchor");
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 1).expect("transitions anchor");
    publish_anchored(serving, world.placement(), checkpoint, 0, value(2, 900))
        .expect("checkpoint anchor");

    for index in 0..28 {
        publish_anchored(
            serving,
            world.placement(),
            name(&format!("anchor.{index:02}")),
            0,
            value(index as u8, 64),
        )
        .expect("the remaining anchor slots are free");
    }
    let quiet = serving.physical_mutation_observation();
    assert!(matches!(
        publish_anchored(
            serving,
            world.placement(),
            name("anchor.overflow"),
            0,
            value(5, 64)
        ),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "the refusal wrote"
    );

    // Every anchor keeps advancing within its own slot.
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Commits, 2).expect("second commit");
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 2).expect("second transition");
    publish_anchored(serving, world.placement(), checkpoint, 1, value(3, 900))
        .expect("second checkpoint");
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "mixed 32 anchors");
    for index in 0..28 {
        assert_eq!(
            current(&recovered, name(&format!("anchor.{index:02}"))),
            Some((1, value(index as u8, 64)))
        );
    }
    assert_holds(&recovered, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 2);
    assert_holds(&recovered, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 2);
    assert_eq!(current(&recovered, home), Some((1, value(1, 200))));
    assert_eq!(current(&recovered, checkpoint), Some((2, value(3, 900))));
    assert!(matches!(
        publish_anchored(
            &recovered,
            placement(),
            name("anchor.overflow"),
            0,
            value(5, 64)
        ),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    let _ = recovered.close();
}

#[test]
fn a_journal_first_entry_refuses_a_full_anchor_table() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("journal-table-full")
        .expect("anchor table store");
    let serving = world.serving();
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Commits, 1).expect("commits anchor");
    for index in 0..31 {
        publish_anchored(
            serving,
            world.placement(),
            name(&format!("anchor.{index:02}")),
            0,
            value(index as u8, 64),
        )
        .expect("an anchor slot is free");
    }
    let quiet = serving.physical_mutation_observation();
    assert!(matches!(
        publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 1),
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::AnchorTableFull)
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "the refusal wrote"
    );
    assert_holds(serving, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 0);
    // A journal that already holds its anchor keeps publishing.
    publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Commits, 2).expect("second commit");
    assert_holds(serving, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 2);
}
