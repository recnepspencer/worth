//! Anchored records publish a whole replacement value at the next generation
//! under one durable root decision, refuse stale, oversized and reserved
//! requests before any effect, coexist with the canonical commit journal and
//! survive an abandoned runtime through production recovery.

#[allow(unused_imports)]
use worth_store_testkit_recovery_runtime::production_entry::anchored_records::*;
use {
    worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
    worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};

#[test]
fn anchored_record_names_refuse_the_ordered_journals_and_malformed_names() {
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("reserved-anchor-refusal").unwrap();
    for journal in [RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS] {
        assert!(matches!(
            publish(
                world.serving(),
                world.placement(),
                name(journal),
                0,
                vec![1]
            ),
            Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::ReservedForOrderedJournal)
        ));
    }
    assert_eq!(
        worth_store::physical_runtime::AnchoredRecordName::new("Relational Home"),
        Err(worth_store::physical_runtime::AnchoredRecordNameDenial::Malformed)
    );
    assert_eq!(name(HOME).as_str(), HOME);
}

#[test]
fn an_anchored_record_is_replaced_one_generation_at_a_time() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-in-process")
        .expect("anchored record store");
    let serving = world.serving();
    let checkpoint = name(CHECKPOINT);
    assert_eq!(current(serving, checkpoint), None);
    assert!(matches!(
        publish(serving, world.placement(), checkpoint, 0, Vec::new()),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::EmptyPayload)
    ));

    let first = publish(serving, world.placement(), checkpoint, 0, value(1, 700)).unwrap();
    assert!(matches!(
        first,
        worth_store::physical_runtime::AnchoredRecordPublished::Published(_)
    ));
    assert_eq!(first.version().generation(), 1);
    assert_eq!(current(serving, checkpoint), Some((1, value(1, 700))));

    let quiet = serving.physical_mutation_observation();
    let retry = publish(serving, world.placement(), checkpoint, 0, value(1, 700)).unwrap();
    assert_eq!(
        retry,
        worth_store::physical_runtime::AnchoredRecordPublished::AlreadyPublished(
            first.membership().clone()
        )
    );
    assert!(matches!(
        publish(serving, world.placement(), checkpoint, 0, value(2, 700)),
        Err(
            worth_store::physical_runtime::AnchoredRecordPublicationFailure::GenerationConflict {
                current: 1
            }
        )
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );

    let second = publish(serving, world.placement(), checkpoint, 1, value(2, 900)).unwrap();
    assert!(matches!(
        second,
        worth_store::physical_runtime::AnchoredRecordPublished::Published(_)
    ));
    assert_eq!(second.version().generation(), 2);
    assert_ne!(second.version().record(), first.version().record());
    assert_eq!(current(serving, checkpoint), Some((2, value(2, 900))));

    let quiet = serving.physical_mutation_observation();
    for (expected, payload) in [(0, value(1, 700)), (1, value(3, 10)), (5, value(3, 10))] {
        assert!(matches!(
            publish(serving, world.placement(), checkpoint, expected, payload),
            Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::GenerationConflict { current: 2 })
        ));
    }
    let records = serving.anchored_records().unwrap();
    assert!(matches!(
        records.read(checkpoint, 899),
        Err(
            worth_store::physical_runtime::AnchoredRecordReadFailure::LimitExceeded {
                payload_bytes: 900
            }
        )
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );
    assert_eq!(current(serving, checkpoint), Some((2, value(2, 900))));
}

#[test]
fn the_anchor_table_bounds_the_number_of_anchored_records() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-table")
        .expect("anchored record store");
    let names: Vec<_> = (0..32).map(|i| format!("anchor.{i:02}")).collect();
    for (index, text) in names.iter().enumerate() {
        publish(
            world.serving(),
            world.placement(),
            name(text),
            0,
            value(index as u8, 64),
        )
        .expect("an anchor slot is free");
    }
    assert!(matches!(
        publish(
            world.serving(),
            world.placement(),
            name("anchor.overflow"),
            0,
            value(9, 64)
        ),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    crate::production_entry::anchored_records::capacity::rewrite(
        world.serving(),
        world.placement(),
    );
    let retained = world.retained_root();
    drop(world);
    let serving = serve(retained.path(), "32 anchors through maintenance");
    for (index, text) in names.iter().enumerate() {
        assert_eq!(
            current(&serving, name(text)),
            Some((1, value(index as u8, 64)))
        );
    }
}

#[test]
fn a_multi_megabyte_anchored_record_round_trips() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-large")
        .expect("anchored record store");
    let large = value(0x3c, 3 << 20);
    publish(
        world.serving(),
        world.placement(),
        name(CHECKPOINT),
        0,
        large.clone(),
    )
    .expect("a large publication");
    let (generation, bytes) = current(world.serving(), name(CHECKPOINT)).unwrap();
    assert_eq!(generation, 1);
    assert!(bytes == large, "the large value changed");
}

/// Interleaves journal entries with two anchored records, abandons the
/// runtime without closing it, and requires production recovery to serve the
/// latest durable generation of each and the whole journal, twice.
#[test]
fn anchored_records_and_the_journal_survive_an_abandoned_runtime() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-abandoned")
        .expect("anchored record store");
    let (checkpoint, home) = (name(CHECKPOINT), name(HOME));
    let serving = world.serving();
    publish_journal_entry(serving, world.placement(), 1);
    publish(serving, world.placement(), checkpoint, 0, value(1, 2_000)).unwrap();
    publish_journal_entry(serving, world.placement(), 2);
    publish(serving, world.placement(), home, 0, value(7, 120)).unwrap();
    publish_journal_entry(serving, world.placement(), 3);
    publish(serving, world.placement(), checkpoint, 1, value(2, 30_000)).unwrap();
    publish_journal_entry(serving, world.placement(), 4);
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "abandoned anchored records");
    assert_journal(&recovered, 4);
    assert_eq!(current(&recovered, checkpoint), Some((2, value(2, 30_000))));
    assert_eq!(current(&recovered, home), Some((1, value(7, 120))));
    publish(&recovered, placement(), checkpoint, 2, value(3, 5_000)).unwrap();
    publish_journal_entry(&recovered, placement(), 5);
    let _ = recovered.close();

    let again = serve(retained.path(), "second recovery");
    assert_journal(&again, 5);
    assert_eq!(current(&again, checkpoint), Some((3, value(3, 5_000))));
    assert_eq!(current(&again, home), Some((1, value(7, 120))));
    let _ = again.close();
}

mod owned_publication;
