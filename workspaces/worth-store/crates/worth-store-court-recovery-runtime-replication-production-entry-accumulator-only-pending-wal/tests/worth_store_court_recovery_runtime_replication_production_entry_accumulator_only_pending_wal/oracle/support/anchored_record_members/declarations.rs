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

pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::AnchoredRecordMember,
    worth_store::physical_runtime::AnchoredRecordMemberLocator,
    worth_store::physical_runtime::AnchoredRecordPublicationFailure,
    worth_store::physical_runtime::AnchoredRecordPublished,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {crate::production_entry::anchored_record_members::assert_value, crate::production_entry::anchored_record_members::carried, crate::production_entry::anchored_record_members::content, crate::production_entry::anchored_record_members::new_member, crate::production_entry::anchored_record_members::publish_with_members};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::assert_journal, worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish_journal_entry, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};

pub(crate) const JOURNAL_HEAD: u64 = 2;
pub(crate) const REPLACED: [usize; 3] = [1, 4, 6];

pub(crate) fn first_contents() -> Vec<Vec<u8>> {
    (0..8_u32)
        .map(|index| match index {
            2 => content(index, 120_000),
            5 => content(index, 40_000),
            _ => content(index, 2_000 + index as usize * 700),
        })
        .collect()
}

/// The replaced members: one extent-backed, two inline.
pub(crate) fn second_contents() -> Vec<Vec<u8>> {
    let mut contents = first_contents();
    contents[1] = content(101, 60_000);
    contents[4] = content(104, 3_100);
    contents[6] = content(106, 900);
    contents
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Held {
    Old,
    New,
}

/// A cleanly closed store with journal entries and the first value; returns
/// the first value's member locators.
pub(crate) fn prepared_store() -> (
    worth_store_test_support::TemporaryDirectory,
    Vec<AnchoredRecordMemberLocator>,
) {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-members-oracle")
        .expect("oracle store");
    for position in 1..=JOURNAL_HEAD {
        publish_journal_entry(world.serving(), world.placement(), position);
    }
    let first = publish_with_members(
        world.serving(),
        world.placement(),
        name(CHECKPOINT),
        0,
        value(1, 500),
        first_contents().into_iter().map(new_member).collect(),
    )
    .expect("the first value");
    let root = world.retained_root();
    let _ = world.close();
    (root, first.members().to_vec())
}

pub(crate) fn publish_replacement(
    serving: &ServingPhysicalRuntime,
    first: &[AnchoredRecordMemberLocator],
) -> Result<AnchoredRecordPublished, AnchoredRecordPublicationFailure> {
    let second = second_contents();
    let members: Vec<AnchoredRecordMember> = first
        .iter()
        .enumerate()
        .map(|(index, locator)| match REPLACED.contains(&index) {
            true => new_member(second[index].clone()),
            false => carried(*locator),
        })
        .collect();
    publish_with_members(
        serving,
        placement(),
        name(CHECKPOINT),
        1,
        value(2, 500),
        members,
    )
}

/// Which value the anchor holds, after reading every member of it back.
pub(crate) fn held(serving: &ServingPhysicalRuntime) -> Held {
    let generation = serving
        .anchored_records()
        .unwrap()
        .read(name(CHECKPOINT), 1 << 20)
        .unwrap_or_else(|failure| panic!("the value must read: {failure:?}"))
        .expect("the value exists")
        .version()
        .generation();
    match generation {
        1 => {
            assert_value(
                serving,
                name(CHECKPOINT),
                1,
                &value(1, 500),
                &first_contents(),
            );
            crate::oracle::support::anchored_record_members::Held::Old
        }
        2 => {
            assert_value(
                serving,
                name(CHECKPOINT),
                2,
                &value(2, 500),
                &second_contents(),
            );
            crate::oracle::support::anchored_record_members::Held::New
        }
        other => panic!("unexpected generation {other}"),
    }
}

/// Counts each role's identified operations while the replacement publishes.
pub(crate) fn measure() -> Vec<(MediaOperationRole, u64)> {
    let (store, first) = prepared_store();
    let never = worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, std::num::NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(store.path(), &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish_replacement(&serving, &first).expect("the measured replacement completes");
    let counts = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// What one faulted replacement answered, by the phase it stopped in.
#[derive(Default)]
pub(crate) struct Answers {
    pub(crate) member_batch_refusals: usize,
    pub(crate) anchor_batch_failures: usize,
}

/// One faulted replacement, then production recovery.
pub(crate) fn run_case(
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
    answers: &mut Answers,
) {
    let (store, first) = prepared_store();
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return;
    };
    activations[0].arm().unwrap();
    let outcome = publish_replacement(&serving, &first);
    assert!(activations[0].is_consumed(), "{role:?}#{nth} did not fire");
    let case = format!("{role:?}#{nth} {directive:?}");
    let anchor_unresolved = matches!(
        outcome,
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::Indeterminate(_)
            | worth_store::physical_runtime::AnchoredRecordPublicationFailure::CompletedWithoutRecord)
    );
    if anchor_unresolved && serving.anchored_records().is_ok() {
        assert!(
            matches!(
                publish_replacement(&serving, &first),
                Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::Unresolved)
            ),
            "{case}: an unresolved name admitted a publication"
        );
    }
    let _ = serving.abort();
    let recovered = serve(store.path(), &case);
    let held = held(&recovered);
    match &outcome {
        Ok(_) => assert_eq!(
            held,
            crate::oracle::support::anchored_record_members::Held::New,
            "{case}: acknowledged value lost"
        ),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::MemberWrite {
            ..
        }) => {
            answers.member_batch_refusals += 1;
            assert_eq!(
                held,
                crate::oracle::support::anchored_record_members::Held::Old,
                "{case}: published without its anchor batch"
            );
        }
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::ProvenNoEffect(_)) => {
            answers.anchor_batch_failures += 1;
            assert_eq!(
                held,
                crate::oracle::support::anchored_record_members::Held::Old,
                "{case}: a no-effect value appeared"
            );
        }
        Err(_) => answers.anchor_batch_failures += 1,
    }
    assert_journal(&recovered, JOURNAL_HEAD);
    let republished = publish_replacement(&recovered, &first)
        .unwrap_or_else(|failure| panic!("{case}: republication: {failure:?}"));
    assert_eq!(
        matches!(
            republished,
            worth_store::physical_runtime::AnchoredRecordPublished::AlreadyPublished(_)
        ),
        held == crate::oracle::support::anchored_record_members::Held::New,
        "{case}: republication answer"
    );
    assert_eq!(
        crate::oracle::support::anchored_record_members::held(&recovered),
        crate::oracle::support::anchored_record_members::Held::New,
        "{case}: republished value"
    );
    let _ = recovered.close();
    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_eq!(
        crate::oracle::support::anchored_record_members::held(&again),
        crate::oracle::support::anchored_record_members::Held::New,
        "{case}: second recovery"
    );
    assert_journal(&again, JOURNAL_HEAD);
    let _ = again.close();
}
