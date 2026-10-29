//! Anchored values with members: a value and its new member records become
//! current in one anchor advance, a replacement carries unchanged members
//! without rewriting them, membership is checked against the current value
//! on every read and carry, and every refusal precedes any effect.

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store::physical_runtime::certification::MediaOperationRole};
pub(crate) use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::AnchoredRecordMember, worth_store::physical_runtime::AnchoredRecordMemberLocator, worth_store::physical_runtime::AnchoredRecordMemberReadFailure, worth_store::physical_runtime::AnchoredRecordName, worth_store::physical_runtime::AnchoredRecordPublication, worth_store::physical_runtime::AnchoredRecordPublicationFailure, worth_store::physical_runtime::AnchoredRecordPublished, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_store_testkit_recovery_runtime::production_entry::anchored_records};




pub(crate) const READ_LIMIT: u64 = 8 << 20;
pub(crate) const MANIFEST_BYTES: usize = 900;

/// Deterministic member content, distinct per `seed`.
pub(crate) fn content(seed: u32, length: usize) -> Vec<u8> {
    let salt = sha2::Sha256::digest(seed.to_le_bytes());
    (0..length)
        .map(|index| salt[index % 32] ^ (index as u8).rotate_left(3) ^ (index >> 9) as u8)
        .collect()
}

pub(crate) fn new_member(bytes: Vec<u8>) -> AnchoredRecordMember {
    worth_store::physical_runtime::AnchoredRecordMember::New {
        key: sha2::Sha256::digest(&bytes).into(),
        bytes,
    }
}

pub(crate) fn carried(locator: AnchoredRecordMemberLocator) -> AnchoredRecordMember {
    worth_store::physical_runtime::AnchoredRecordMember::Carried { locator }
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
            worth_store::physical_runtime::AnchoredRecordPublication::new(
                name,
                expected_generation,
                payload,
                placement,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
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
    counters.completed_bytes_for(worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite)
        + counters.completed_bytes_for(worth_store::physical_runtime::certification::MediaOperationRole::Append)
}

/// Two hundred members of mixed sizes up to 2 MiB.
pub(crate) fn initial_members() -> Vec<Vec<u8>> {
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