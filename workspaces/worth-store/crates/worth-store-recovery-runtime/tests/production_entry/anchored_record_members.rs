//! Anchored values with members: a value and its new member records become
//! current in one anchor advance, a replacement carries unchanged members
//! without rewriting them, membership is checked against the current value
//! on every read and carry, and every refusal precedes any effect.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, AnchoredRecordMember, AnchoredRecordMemberLocator,
    AnchoredRecordMemberReadFailure, AnchoredRecordName, AnchoredRecordPublication,
    AnchoredRecordPublicationFailure, AnchoredRecordPublished, ServingPhysicalRuntime,
};

use super::anchored_records::{name, value, CHECKPOINT};
use super::*;

#[path = "anchored_record_members/oracle.rs"]
mod oracle;
#[path = "anchored_record_members/recovery.rs"]
mod recovery;

const READ_LIMIT: u64 = 8 << 20;
const MANIFEST_BYTES: usize = 900;

/// Deterministic member content, distinct per `seed`.
pub(crate) fn content(seed: u32, length: usize) -> Vec<u8> {
    let salt = Sha256::digest(seed.to_le_bytes());
    (0..length)
        .map(|index| salt[index % 32] ^ (index as u8).rotate_left(3) ^ (index >> 9) as u8)
        .collect()
}

pub(crate) fn new_member(bytes: Vec<u8>) -> AnchoredRecordMember {
    AnchoredRecordMember::New {
        key: Sha256::digest(&bytes).into(),
        bytes,
    }
}

pub(crate) fn carried(locator: AnchoredRecordMemberLocator) -> AnchoredRecordMember {
    AnchoredRecordMember::Carried { locator }
}

pub(crate) fn publish_with_members(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    name: AnchoredRecordName,
    expected_generation: u64,
    payload: Vec<u8>,
    members: Vec<AnchoredRecordMember>,
) -> Result<AnchoredRecordPublished, AnchoredRecordPublicationFailure> {
    serving
        .anchored_records()
        .expect("anchored record access")
        .publish(
            AnchoredRecordPublication::new(
                name,
                expected_generation,
                payload,
                placement,
                PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            )
            .with_members(members),
        )
}

/// Requires `name` to hold `generation` with `payload` and exactly these
/// member contents in order, every one read back through its locator.
/// Returns the current locators.
pub(crate) fn assert_value(
    serving: &ServingPhysicalRuntime,
    name: AnchoredRecordName,
    generation: u64,
    payload: &[u8],
    members: &[Vec<u8>],
) -> Vec<AnchoredRecordMemberLocator> {
    let records = serving.anchored_records().expect("anchored record access");
    let value = records
        .read(name, READ_LIMIT)
        .unwrap_or_else(|failure| panic!("the value must read: {failure:?}"))
        .expect("the value exists");
    assert_eq!(value.version().generation(), generation);
    assert!(value.bytes() == payload, "the payload changed");
    assert_eq!(value.members().len(), members.len());
    for (index, (locator, expected)) in value.members().iter().zip(members).enumerate() {
        assert_eq!(locator.length(), expected.len() as u64, "member {index}");
        let bytes = records
            .read_member(name, *locator, READ_LIMIT)
            .unwrap_or_else(|failure| panic!("member {index} must read: {failure:?}"));
        assert!(bytes == *expected, "member {index} changed");
    }
    value.members().to_vec()
}

/// Bytes the media completed writing, by positioned write or append.
pub(crate) fn written_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    let counters = serving.media_counters();
    counters.completed_bytes_for(MediaOperationRole::PositionedWrite)
        + counters.completed_bytes_for(MediaOperationRole::Append)
}

/// Two hundred members of mixed sizes up to 2 MiB.
fn initial_members() -> Vec<Vec<u8>> {
    (0..200_u32)
        .map(|index| {
            let length = match index {
                _ if index % 50 == 0 => 2 << 20,
                _ if index % 7 == 0 => (200 << 10) + index as usize * 13,
                _ => 1_000 + (index as usize * 7_919) % 60_000,
            };
            content(index, length)
        })
        .collect()
}

#[test]
fn two_hundred_members_publish_read_back_and_carry_forward_without_rewriting() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-members")
        .expect("anchored record store");
    let (serving, checkpoint) = (world.serving(), name(CHECKPOINT));
    let first_contents = initial_members();
    let total: u64 = first_contents.iter().map(|bytes| bytes.len() as u64).sum();
    let before = written_bytes(serving);
    let first = publish_with_members(
        serving,
        world.placement(),
        checkpoint,
        0,
        value(1, MANIFEST_BYTES),
        first_contents.iter().cloned().map(new_member).collect(),
    )
    .expect("the first value publishes");
    assert!(matches!(first, AnchoredRecordPublished::Published(_)));
    assert!(
        written_bytes(serving) - before >= total,
        "the first value's members were written"
    );
    let first_locators = assert_value(
        serving,
        checkpoint,
        1,
        &value(1, MANIFEST_BYTES),
        &first_contents,
    );
    assert_eq!(first.members(), first_locators.as_slice());

    // Replace three members; carry the other 197.
    let replaced = [5_usize, 77, 133];
    let mut second_contents = first_contents.clone();
    let mut request = Vec::new();
    for (index, locator) in first_locators.iter().enumerate() {
        if replaced.contains(&index) {
            second_contents[index] = content(1_000 + index as u32, 9_000 + index);
            request.push(new_member(second_contents[index].clone()));
        } else {
            request.push(carried(*locator));
        }
    }
    let new_bytes: u64 = replaced
        .iter()
        .map(|index| second_contents[*index].len() as u64)
        .sum();
    let carried_bytes = total
        - replaced
            .iter()
            .map(|i| first_contents[*i].len() as u64)
            .sum::<u64>();
    let before = written_bytes(serving);
    let second = publish_with_members(
        serving,
        world.placement(),
        checkpoint,
        1,
        value(2, MANIFEST_BYTES),
        request,
    )
    .expect("the replacement publishes");
    let replacement_written = written_bytes(serving) - before;
    assert!(
        replacement_written < new_bytes + (4 << 20) && replacement_written < carried_bytes / 4,
        "the replacement wrote {replacement_written} bytes for {new_bytes} new member bytes"
    );
    let second_locators = assert_value(
        serving,
        checkpoint,
        2,
        &value(2, MANIFEST_BYTES),
        &second_contents,
    );
    assert_eq!(second.members(), second_locators.as_slice());
    for (index, (old, new)) in first_locators.iter().zip(&second_locators).enumerate() {
        assert_eq!(old == new, !replaced.contains(&index), "member {index}");
    }

    // A replaced member is no longer current and never reads as one.
    let records = serving.anchored_records().unwrap();
    for index in replaced {
        assert!(matches!(
            records.read_member(checkpoint, first_locators[index], READ_LIMIT),
            Err(AnchoredRecordMemberReadFailure::NotCurrentMember)
        ));
    }
    assert!(matches!(
        records.read_member(checkpoint, second_locators[0], 10),
        Err(AnchoredRecordMemberReadFailure::LimitExceeded { .. })
    ));

    // An identical retry is answered from durable state and writes nothing.
    let quiet = serving.physical_mutation_observation();
    let before = written_bytes(serving);
    let retry_members = second_contents
        .iter()
        .zip(&first_locators)
        .enumerate()
        .map(
            |(index, (bytes, locator))| match replaced.contains(&index) {
                true => new_member(bytes.clone()),
                false => carried(*locator),
            },
        )
        .collect();
    let retry = publish_with_members(
        serving,
        world.placement(),
        checkpoint,
        1,
        value(2, MANIFEST_BYTES),
        retry_members,
    )
    .expect("an identical retry");
    assert_eq!(
        retry,
        AnchoredRecordPublished::AlreadyPublished(second.membership().clone())
    );
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "the retry wrote"
    );
    assert_eq!(written_bytes(serving), before, "the retry wrote");
    let _ = world.close();
}

#[test]
fn members_are_admitted_against_the_current_value_before_any_effect() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-member-refusals")
        .expect("anchored record store");
    let (serving, checkpoint) = (world.serving(), name(CHECKPOINT));
    let first = publish_with_members(
        serving,
        world.placement(),
        checkpoint,
        0,
        value(1, 300),
        (0..4)
            .map(|seed| new_member(content(seed, 2_000)))
            .collect(),
    )
    .expect("the first value");
    let old = first.members().to_vec();
    publish_with_members(
        serving,
        world.placement(),
        checkpoint,
        1,
        value(2, 300),
        vec![carried(old[0]), new_member(content(9, 1_500))],
    )
    .expect("the replacement drops three members");

    let quiet = serving.physical_mutation_observation();
    let before = written_bytes(serving);
    let refuse = |expected: u64, members: Vec<AnchoredRecordMember>| {
        publish_with_members(
            serving,
            world.placement(),
            checkpoint,
            expected,
            value(3, 300),
            members,
        )
    };
    // A member of the replaced value is no longer current.
    assert!(matches!(
        refuse(2, vec![new_member(content(20, 100)), carried(old[2])]),
        Err(AnchoredRecordPublicationFailure::CarriedMemberNotCurrent { index: 1 })
    ));
    // A key that is not the bytes' digest.
    let forged = AnchoredRecordMember::New {
        key: [7; 32],
        bytes: content(21, 100),
    };
    assert!(matches!(
        refuse(2, vec![forged]),
        Err(AnchoredRecordPublicationFailure::MemberKeyMismatch { index: 0 })
    ));
    // A stale generation, even with valid members.
    assert!(matches!(
        refuse(1, vec![carried(old[0])]),
        Err(AnchoredRecordPublicationFailure::GenerationConflict { current: 2 })
    ));
    // A carried member under a name that has no value.
    assert!(matches!(
        publish_with_members(
            serving,
            world.placement(),
            name("relational.home"),
            0,
            value(4, 100),
            vec![carried(old[0])],
        ),
        Err(AnchoredRecordPublicationFailure::CarriedMemberNotCurrent { index: 0 })
    ));
    let records = serving.anchored_records().unwrap();
    assert!(matches!(
        records.read_member(name("relational.home"), old[0], READ_LIMIT),
        Err(AnchoredRecordMemberReadFailure::NotCurrentMember)
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );
    assert_eq!(written_bytes(serving), before, "refusals wrote");
    assert_value(
        serving,
        checkpoint,
        2,
        &value(2, 300),
        &[content(0, 2_000), content(9, 1_500)],
    );
    let _ = world.close();
}
