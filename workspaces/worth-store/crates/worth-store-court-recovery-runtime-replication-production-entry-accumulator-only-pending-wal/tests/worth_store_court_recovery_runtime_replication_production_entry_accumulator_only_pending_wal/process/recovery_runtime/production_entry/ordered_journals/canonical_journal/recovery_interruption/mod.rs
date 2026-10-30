//! A recovery killed at each of its own stages leaves a store the next
//! recovery settles to the same journal: the interrupted publication rolls
//! forward exactly once, and nothing the first attempt did is repeated or lost.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::*;

use {
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::assert_journal_holds,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::launch,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::payload,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::placement,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::publication,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::serve,
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::ENTRIES,
};

#[test]
fn a_recovery_killed_at_each_of_its_stages_settles_to_the_same_journal() {
    let mut reached_any = false;
    for stage in STAGES {
        let (root, _marker) = launch(worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AfterWalDurability);
        reached_any |= kill_recovery_at(&root, stage);
        let serving = serve(&root, stage.label());
        assert_journal_holds(&serving, ENTRIES);
        publication(&serving, placement(), ENTRIES + 1, payload(ENTRIES + 1))
            .expect("publication after an interrupted recovery");
        serving.close();
        let serving = serve(&root, "recovery after the interrupted one");
        assert_journal_holds(&serving, ENTRIES + 1);
        serving.close();
    }
    assert!(reached_any, "no recovery stage was interrupted");
}

#[test]
fn recovery_interruption_child() {
    worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::dispatch_recovery_interruption_child();
}
