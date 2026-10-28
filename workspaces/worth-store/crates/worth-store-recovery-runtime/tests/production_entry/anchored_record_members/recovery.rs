//! Anchored values with members survive abandoned runtimes through production
//! recovery, and a member whose durable bytes are damaged refuses with a
//! typed failure instead of reading as content or as absence.

use std::path::Path;

use worth_store::physical_runtime::AnchoredRecordMemberReadFailure;

use super::{assert_value, carried, content, new_member, publish_with_members};
use crate::anchored_records::{assert_journal, name, publish_journal_entry, value, CHECKPOINT};
use crate::canonical_journal_oracle::{placement, serve};
use crate::*;

/// Thirty members, inline and extent-backed, small enough that recovery
/// replays them from the WAL tail without a Store checkpoint.
fn first_contents() -> Vec<Vec<u8>> {
    (0..30_u32)
        .map(|index| match index {
            0 => content(index, 300_000),
            _ if index % 4 == 0 => content(index, 40_000 + index as usize),
            _ => content(index, 500 + index as usize * 311),
        })
        .collect()
}

#[test]
fn members_survive_abandoned_runtimes_and_recovery_twice() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-members-abandoned")
        .expect("anchored record store");
    let checkpoint = name(CHECKPOINT);
    let first = first_contents();
    publish_journal_entry(world.serving(), world.placement(), 1);
    let published = publish_with_members(
        world.serving(),
        world.placement(),
        checkpoint,
        0,
        value(1, 640),
        first.iter().cloned().map(new_member).collect(),
    )
    .expect("the first value");
    publish_journal_entry(world.serving(), world.placement(), 2);
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "first abandoned runtime");
    assert_journal(&recovered, 2);
    let locators = assert_value(&recovered, checkpoint, 1, &value(1, 640), &first);
    assert_eq!(locators, published.members());
    let mut second = first.clone();
    let mut request: Vec<_> = locators.iter().copied().map(carried).collect();
    for index in [0_usize, 17] {
        second[index] = content(500 + index as u32, 4_000);
        request[index] = new_member(second[index].clone());
    }
    publish_with_members(
        &recovered,
        placement(),
        checkpoint,
        1,
        value(2, 640),
        request,
    )
    .expect("a carrying replacement after recovery");
    publish_journal_entry(&recovered, placement(), 3);
    let _ = recovered.abort();

    let again = serve(retained.path(), "second abandoned runtime");
    assert_journal(&again, 3);
    assert_value(&again, checkpoint, 2, &value(2, 640), &second);
    let _ = again.close();
    let reopened = serve(retained.path(), "clean reopen");
    assert_value(&reopened, checkpoint, 2, &value(2, 640), &second);
    let _ = reopened.close();
}

#[test]
fn a_damaged_member_refuses_typed_and_never_reads_as_absent() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-member-damage")
        .expect("anchored record store");
    let checkpoint = name(CHECKPOINT);
    // The damaged members are extent-backed, so damage to them leaves the
    // anchored record's own page intact.
    let contents = [content(1, 100_000), content(2, 1 << 20), content(3, 5_000)];
    publish_with_members(
        world.serving(),
        world.placement(),
        checkpoint,
        0,
        value(9, 200),
        contents.iter().cloned().map(new_member).collect(),
    )
    .expect("the value");
    // A Store checkpoint makes the record artifacts, not the WAL tail, the
    // only durable copy recovery serves the members from.
    store_checkpoint(world.serving());
    let root = world.retained_root();
    let _ = world.close();
    for damaged in [&contents[0], &contents[1]] {
        assert!(
            flip_every_record_copy(root.path(), damaged) > 0,
            "the member's bytes were found in a record artifact"
        );
    }

    let recovered = serve(root.path(), "damaged members");
    let records = recovered.anchored_records().unwrap();
    let value = records.read(checkpoint, 1 << 20).unwrap().unwrap();
    for (index, locator) in value.members().iter().enumerate() {
        let outcome = records.read_member(checkpoint, *locator, 4 << 20);
        if index == 2 {
            assert!(outcome.unwrap() == contents[2], "the intact member changed");
            continue;
        }
        assert!(
            matches!(
                outcome,
                Err(AnchoredRecordMemberReadFailure::Damaged(_)
                    | AnchoredRecordMemberReadFailure::Stream(_)
                    | AnchoredRecordMemberReadFailure::Open(_))
            ),
            "member {index} answered {outcome:?}"
        );
    }
    let _ = recovered.close();
}

/// Completes one fuzzy Store physical checkpoint.
pub(crate) fn store_checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xa7; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(60_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("the checkpoint must start");
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "{outcome:?}"
    );
}

/// Flips one byte in the middle of every copy of `member` held by a record
/// artifact (not the WAL); returns the number of copies found.
fn flip_every_record_copy(root: &Path, member: &[u8]) -> usize {
    let probe = &member[member.len() / 2..member.len() / 2 + 64];
    let mut flipped = 0;
    let mut pending = vec![root.join("families").join("records")];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let mut bytes = std::fs::read(&path).unwrap();
            let Some(at) = bytes
                .windows(probe.len())
                .position(|window| window == probe)
            else {
                continue;
            };
            bytes[at + 32] ^= 0x5a;
            std::fs::write(&path, bytes).unwrap();
            flipped += 1;
        }
    }
    flipped
}
