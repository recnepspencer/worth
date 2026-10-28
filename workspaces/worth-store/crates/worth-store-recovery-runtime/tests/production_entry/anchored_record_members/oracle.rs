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

use std::num::NonZeroU64;

use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordPublicationFailure,
    AnchoredRecordPublished, ServingPhysicalRuntime,
};

use super::{assert_value, carried, content, new_member, publish_with_members};
use crate::anchored_records::{assert_journal, name, publish_journal_entry, value, CHECKPOINT};
use crate::canonical_journal_oracle::{open_with_faults, placement, serve};
use crate::*;

const JOURNAL_HEAD: u64 = 2;
const REPLACED: [usize; 3] = [1, 4, 6];

fn first_contents() -> Vec<Vec<u8>> {
    (0..8_u32)
        .map(|index| match index {
            2 => content(index, 120_000),
            5 => content(index, 40_000),
            _ => content(index, 2_000 + index as usize * 700),
        })
        .collect()
}

/// The replaced members: one extent-backed, two inline.
fn second_contents() -> Vec<Vec<u8>> {
    let mut contents = first_contents();
    contents[1] = content(101, 60_000);
    contents[4] = content(104, 3_100);
    contents[6] = content(106, 900);
    contents
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Old,
    New,
}

/// A cleanly closed store with journal entries and the first value; returns
/// the first value's member locators.
fn prepared_store() -> (
    worth_store_test_support::TemporaryDirectory,
    Vec<AnchoredRecordMemberLocator>,
) {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-members-oracle")
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

fn publish_replacement(
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
fn held(serving: &ServingPhysicalRuntime) -> Held {
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
            Held::Old
        }
        2 => {
            assert_value(
                serving,
                name(CHECKPOINT),
                2,
                &value(2, 500),
                &second_contents(),
            );
            Held::New
        }
        other => panic!("unexpected generation {other}"),
    }
}

/// Counts each role's identified operations while the replacement publishes.
fn measure() -> Vec<(MediaOperationRole, u64)> {
    let (store, first) = prepared_store();
    let never = MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(store.path(), &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish_replacement(&serving, &first).expect("the measured replacement completes");
    let counts = MediaOperationRole::ALL
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
struct Answers {
    member_batch_refusals: usize,
    anchor_batch_failures: usize,
}

/// One faulted replacement, then production recovery.
fn run_case(
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
        Err(AnchoredRecordPublicationFailure::Indeterminate(_)
            | AnchoredRecordPublicationFailure::CompletedWithoutRecord)
    );
    if anchor_unresolved && serving.anchored_records().is_ok() {
        assert!(
            matches!(
                publish_replacement(&serving, &first),
                Err(AnchoredRecordPublicationFailure::Unresolved)
            ),
            "{case}: an unresolved name admitted a publication"
        );
    }
    let _ = serving.abort();
    let recovered = serve(store.path(), &case);
    let held = held(&recovered);
    match &outcome {
        Ok(_) => assert_eq!(held, Held::New, "{case}: acknowledged value lost"),
        Err(AnchoredRecordPublicationFailure::MemberWrite { .. }) => {
            answers.member_batch_refusals += 1;
            assert_eq!(
                held,
                Held::Old,
                "{case}: published without its anchor batch"
            );
        }
        Err(AnchoredRecordPublicationFailure::ProvenNoEffect(_)) => {
            answers.anchor_batch_failures += 1;
            assert_eq!(held, Held::Old, "{case}: a no-effect value appeared");
        }
        Err(_) => answers.anchor_batch_failures += 1,
    }
    assert_journal(&recovered, JOURNAL_HEAD);
    let republished = publish_replacement(&recovered, &first)
        .unwrap_or_else(|failure| panic!("{case}: republication: {failure:?}"));
    assert_eq!(
        matches!(republished, AnchoredRecordPublished::AlreadyPublished(_)),
        held == Held::New,
        "{case}: republication answer"
    );
    assert_eq!(
        self::held(&recovered),
        Held::New,
        "{case}: republished value"
    );
    let _ = recovered.close();
    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_eq!(self::held(&again), Held::New, "{case}: second recovery");
    assert_journal(&again, JOURNAL_HEAD);
    let _ = again.close();
}

#[test]
fn every_media_operation_of_a_member_carrying_replacement_recovers_to_one_whole_value() {
    let counts = measure();
    assert!(
        !counts.is_empty(),
        "the replacement performed media operations"
    );
    let mut answers = Answers::default();
    for (role, count) in counts {
        for nth in 1..=count {
            let nth = NonZeroU64::new(nth).unwrap();
            for directive in [
                MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
                MediaFaultDirective::IndeterminateAfterEffect,
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
