//! Two ordered journals in one home keep independent contiguous sequences:
//! interleaved publications read, range and resolve per journal, the next
//! position is each journal's own head plus one, neither journal ever answers
//! with the other's entry, and both survive abandoned runtimes.

#[allow(unused_imports)] use crate::production_entry::ordered_journals::two_journals::*;
use {sha2::Digest, sha2::Sha256};
use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::OrderedJournal, worth_store::physical_runtime::OrderedJournalName, worth_store::physical_runtime::OrderedJournalNameDenial, worth_store::physical_runtime::OrderedJournalPublication, worth_store::physical_runtime::OrderedJournalPublicationFailure, worth_store::physical_runtime::OrderedJournalPublished, worth_store::physical_runtime::OrderedJournalResolution, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS, worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn two_journals_keep_independent_sequences_reads_and_resolutions() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("two-journals")
        .expect("journal store");
    let serving = world.serving();
    for journal in crate::production_entry::ordered_journals::two_journals::Journal::BOTH {
        assert_holds(serving, journal, 0);
    }
    publish_script(serving, world.placement());
    assert_holds(serving, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 4);
    assert_holds(serving, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 5);

    // Each journal admits only its own head plus one; refusals write nothing.
    let quiet = serving.physical_mutation_observation();
    for (journal, requested, head) in [(crate::production_entry::ordered_journals::two_journals::Journal::Commits, 6, 4), (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 5 + 2, 5)] {
        assert!(matches!(
            publish(serving, world.placement(), journal, requested),
            Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::PositionGap { head: observed, requested: r })
                if observed == head && r == requested
        ));
    }
    assert!(matches!(
        publish(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Commits, 3),
        Ok(worth_store::physical_runtime::OrderedJournalPublished::AlreadyCommitted(entry)) if entry.position() == 3
    ));
    // The other journal's entry at an occupied position is a conflict.
    let foreign = crate::production_entry::ordered_journals::two_journals::Journal::Commits.payload(2);
    assert!(matches!(
        publish_bytes(serving, world.placement(), crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 2, foreign),
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::PositionConflict(entry)) if entry.position() == 2
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );

    // The named door reaches the same journals as the named accessors.
    for (text, journal) in [
        (RELATIONAL_CANONICAL_COMMITS, crate::production_entry::ordered_journals::two_journals::Journal::Commits),
        (RELATIONAL_BRANCH_TRANSITIONS, crate::production_entry::ordered_journals::two_journals::Journal::Transitions),
    ] {
        let name = worth_store::physical_runtime::OrderedJournalName::new(text).expect("a registered journal");
        let by_name = serving.ordered_journal(name).expect("journal access");
        assert_eq!(by_name.name().as_str(), text);
        assert_eq!(by_name.head().unwrap(), journal.of(serving).head().unwrap());
    }
    let quiet = serving.physical_mutation_observation();
    assert!(matches!(
        serving.ordered_journal(worth_store::physical_runtime::OrderedJournalName::new("unregistered.journal").unwrap()),
        Err(worth_store::physical_runtime::OrderedJournalDenial::Unregistered)
    ));
    assert_eq!(serving.physical_mutation_observation(), quiet);
    assert_eq!(
        worth_store::physical_runtime::OrderedJournalName::new("Relational Commits"),
        Err(worth_store::physical_runtime::OrderedJournalNameDenial::Malformed)
    );
}

/// Both journals recover from an abandoned runtime, keep publishing, and
/// recover again from a second abandoned runtime.
#[test]
fn two_journals_survive_abandoned_runtimes_twice() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("two-journals-abandoned")
        .expect("journal store");
    publish_script(world.serving(), world.placement());
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "first abandoned runtime");
    assert_holds(&recovered, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 4);
    assert_holds(&recovered, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 5);
    for (journal, position) in [(crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 6), (crate::production_entry::ordered_journals::two_journals::Journal::Commits, 5)] {
        publish(&recovered, placement(), journal, position).expect("publication after recovery");
    }
    let _ = recovered.abort();

    let again = serve(retained.path(), "second abandoned runtime");
    assert_holds(&again, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 5);
    assert_holds(&again, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 6);
    for (journal, position) in [(crate::production_entry::ordered_journals::two_journals::Journal::Commits, 6), (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 7)] {
        publish(&again, placement(), journal, position).expect("publication after recovery");
    }
    let _ = again.close();

    let reopened = serve(retained.path(), "clean reopen");
    assert_holds(&reopened, crate::production_entry::ordered_journals::two_journals::Journal::Commits, 6);
    assert_holds(&reopened, crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 7);
    let _ = reopened.close();
}
