//! Crash-consistency oracle for one member-carrying anchored replacement.
//!
//! The replacement writes three new members (inline and extent-backed) in a
//! member batch and carries five, then advances the anchor in its own batch.
//! The oracle counts the media operations of every role the replacement
//! performs, reruns it once per (role, ordinal, directive) with a fault at
//! exactly that operation, abandons the runtime, recovers in the production
//! path and requires: the anchor holds exactly the old or the new value with
//! every one of its members readable and intact; an acknowledged value is
//! never lost; a refusal before the anchor batch (proven no effect, or a
//! member batch that did not complete) never shows the new value; the journal
//! beside it is unchanged; the replacement then publishes or answers
//! `AlreadyPublished`; and a second recovery agrees.

#[allow(unused_imports)] use crate::production_entry::anchored_record_members::oracle::*;
use {std::num::NonZeroU64};

use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaOperationRole};
use {worth_store::physical_runtime::AnchoredRecordMember, worth_store::physical_runtime::AnchoredRecordMemberLocator, worth_store::physical_runtime::AnchoredRecordPublicationFailure, worth_store::physical_runtime::AnchoredRecordPublished, worth_store::physical_runtime::ServingPhysicalRuntime};

use {crate::production_entry::anchored_record_members::assert_value, crate::production_entry::anchored_record_members::carried, crate::production_entry::anchored_record_members::content, crate::production_entry::anchored_record_members::new_member, crate::production_entry::anchored_record_members::publish_with_members};
use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::assert_journal, worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish_journal_entry, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT};
use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_store_testkit_recovery_runtime::production_entry::anchored_records};

#[test]
fn every_media_operation_of_a_member_carrying_replacement_recovers_to_one_whole_value() {
    let counts = measure();
    assert!(
        !counts.is_empty(),
        "the replacement performed media operations"
    );
    let mut answers = crate::production_entry::anchored_record_members::oracle::Answers::default();
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
                run_case(role, nth, directive, &mut answers);
            }
        }
    }
    assert!(
        answers.member_batch_refusals > 0 && answers.anchor_batch_failures > 0,
        "the sweep stopped publications in both the member batch and the anchor batch"
    );
}
