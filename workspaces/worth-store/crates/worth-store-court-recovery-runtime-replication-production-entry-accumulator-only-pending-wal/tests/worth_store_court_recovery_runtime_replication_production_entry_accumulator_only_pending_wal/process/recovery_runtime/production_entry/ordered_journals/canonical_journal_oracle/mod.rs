//! Crash-consistency oracle for the canonical commit journal.
//!
//! The reference is a memory-only Relational runtime's real canonical commit
//! artifacts. For one target publication, the oracle first counts the media
//! operations of every role that publication performs, then reruns it once per
//! (role, ordinal, directive) with a fault at exactly that operation, abandons
//! the runtime, recovers in the production path and requires: the journal is a
//! prefix of the reference holding every acknowledged entry; each entry
//! readmits through Relational; `resolve` agrees with the prefix; the suffix
//! publishes; and a second recovery reaches the same journal.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::*;

#[test]
fn every_media_operation_of_a_publication_recovers_to_a_reference_prefix() {
    let reference = worth_store_testkit_recovery_runtime::ordered_journals::canonical_journal_oracle::relational_reference::RelationalReference::generate(SEED, COMMITS);
    let artifacts = reference.artifacts.as_slice();
    let mut cases = Vec::new();
    for target in TARGETS {
        let counts = measure(artifacts, target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
        for (role, count) in counts {
            for nth in 1..=count {
                let nth = std::num::NonZeroU64::new(nth).unwrap();
                for directive in [
                    worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                        kind: std::io::ErrorKind::Other,
                        raw_os_error: None,
                    },
                    worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
                ] {
                    cases.push((target, role, nth, directive));
                }
            }
        }
    }
    let scheduled = cases.len();
    let outcomes = worth_store_test_support::sweep::run(cases, |(target, role, nth, directive)| {
        run_case(artifacts, target, role, nth, directive);
    });
    assert_eq!(
        outcomes.len(),
        scheduled,
        "scheduled journal cases were lost"
    );
    drop(reference);
}
