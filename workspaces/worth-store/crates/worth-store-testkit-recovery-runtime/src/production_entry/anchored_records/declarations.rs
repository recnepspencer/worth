//! Anchored records publish a whole replacement value at the next generation
//! under one durable root decision, refuse stale, oversized and reserved
//! requests before any effect, coexist with the canonical commit journal and
//! survive an abandoned runtime through production recovery.

pub use {sha2::Digest, sha2::Sha256};
pub use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AnchoredRecordName,
    worth_store::physical_runtime::AnchoredRecordNameDenial,
    worth_store::physical_runtime::AnchoredRecordPublication,
    worth_store::physical_runtime::AnchoredRecordPublicationFailure,
    worth_store::physical_runtime::AnchoredRecordPublished,
    worth_store::physical_runtime::AnchoredRecordReadFailure,
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
    worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
};

pub use {
    crate::production_entry::anchored_records,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub use {
    crate::production_entry::ordered_journals::canonical_journal_oracle::placement,
    crate::production_entry::ordered_journals::canonical_journal_oracle::serve,
};

pub const CHECKPOINT: &str = "relational.checkpoint";
pub const HOME: &str = "relational.home";
pub const READ_LIMIT: u64 = 8 << 20;

pub fn name(text: &str) -> AnchoredRecordName {
    worth_store::physical_runtime::AnchoredRecordName::new(text)
        .expect("a valid anchored record name")
}

/// Deterministic bytes distinct per `seed`.
pub fn value(seed: u8, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| seed ^ (index as u8).rotate_left(5) ^ (index >> 8) as u8)
        .collect()
}

pub fn publish(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    name: AnchoredRecordName,
    expected_generation: u64,
    payload: Vec<u8>,
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
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
}

/// The current generation and bytes of `name`; damage fails the test.
pub fn current(
    serving: &ServingPhysicalRuntime,
    name: AnchoredRecordName,
) -> Option<(u64, Vec<u8>)> {
    let record = serving
        .anchored_records()
        .expect("anchored record access")
        .read(name, READ_LIMIT)
        .unwrap_or_else(|failure| panic!("{} must read: {failure:?}", name.as_str()))?;
    let digest: [u8; 32] = sha2::Sha256::digest(record.bytes()).into();
    assert_eq!(record.version().payload_sha256(), digest);
    assert_eq!(
        record.version().payload_bytes(),
        record.bytes().len() as u64
    );
    Some((record.version().generation(), record.into_bytes()))
}

pub fn journal_payload(position: u64) -> Vec<u8> {
    value(0x80 | position as u8, 300 + position as usize * 211)
}

pub fn publish_journal_entry(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    position: u64,
) {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(
            worth_store::physical_runtime::OrderedJournalPublication::new(
                position,
                journal_payload(position),
                placement,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .expect("journal publication");
}

/// Requires the journal to hold exactly entries `1..=head`.
pub fn assert_journal(serving: &ServingPhysicalRuntime, head: u64) {
    let journal = serving.canonical_commits().expect("journal access");
    assert_eq!(
        journal.head().unwrap().map_or(0, |entry| entry.position()),
        head
    );
    for position in 1..=head {
        let entry = journal.read(position, READ_LIMIT).expect("journal read");
        assert!(
            entry.bytes() == journal_payload(position),
            "entry {position} changed"
        );
    }
}
