//! Anchored records publish a whole replacement value at the next generation
//! under one durable root decision, refuse stale, oversized and reserved
//! requests before any effect, coexist with the canonical commit journal and
//! survive an abandoned runtime through production recovery.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, AnchoredRecordName, AnchoredRecordNameDenial,
    AnchoredRecordPublication, AnchoredRecordPublicationFailure, AnchoredRecordPublished,
    AnchoredRecordReadFailure, CanonicalCommitPublication, ServingPhysicalRuntime,
    RELATIONAL_CANONICAL_COMMITS,
};

use super::canonical_journal_oracle::{placement, serve};
use super::*;

pub(crate) const CHECKPOINT: &str = "relational.checkpoint";
const HOME: &str = "relational.home";
const READ_LIMIT: u64 = 8 << 20;

pub(crate) fn name(text: &str) -> AnchoredRecordName {
    AnchoredRecordName::new(text).expect("a valid anchored record name")
}

/// Deterministic bytes distinct per `seed`.
pub(crate) fn value(seed: u8, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| seed ^ (index as u8).rotate_left(5) ^ (index >> 8) as u8)
        .collect()
}

pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    name: AnchoredRecordName,
    expected_generation: u64,
    payload: Vec<u8>,
) -> Result<AnchoredRecordPublished, AnchoredRecordPublicationFailure> {
    serving
        .anchored_records()
        .expect("anchored record access")
        .publish(AnchoredRecordPublication::new(
            name,
            expected_generation,
            payload,
            placement,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
}

/// The current generation and bytes of `name`; damage fails the test.
pub(crate) fn current(
    serving: &ServingPhysicalRuntime,
    name: AnchoredRecordName,
) -> Option<(u64, Vec<u8>)> {
    let record = serving
        .anchored_records()
        .expect("anchored record access")
        .read(name, READ_LIMIT)
        .unwrap_or_else(|failure| panic!("{} must read: {failure:?}", name.as_str()))?;
    let digest: [u8; 32] = Sha256::digest(record.bytes()).into();
    assert_eq!(record.version().payload_sha256(), digest);
    assert_eq!(
        record.version().payload_bytes(),
        record.bytes().len() as u64
    );
    Some((record.version().generation(), record.into_bytes()))
}

pub(crate) fn journal_payload(position: u64) -> Vec<u8> {
    value(0x80 | position as u8, 300 + position as usize * 211)
}

pub(crate) fn publish_journal_entry(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    position: u64,
) {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(CanonicalCommitPublication::new(
            position,
            journal_payload(position),
            placement,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .expect("journal publication");
}

/// Requires the journal to hold exactly entries `1..=head`.
pub(crate) fn assert_journal(serving: &ServingPhysicalRuntime, head: u64) {
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

#[test]
fn anchored_record_names_refuse_the_journal_and_malformed_names() {
    assert_eq!(
        AnchoredRecordName::new(RELATIONAL_CANONICAL_COMMITS),
        Err(AnchoredRecordNameDenial::ReservedForCanonicalJournal)
    );
    assert_eq!(
        AnchoredRecordName::new("Relational Home"),
        Err(AnchoredRecordNameDenial::Malformed)
    );
    assert_eq!(name(HOME).as_str(), HOME);
}

#[test]
fn an_anchored_record_is_replaced_one_generation_at_a_time() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-in-process")
        .expect("anchored record store");
    let serving = world.serving();
    let checkpoint = name(CHECKPOINT);
    assert_eq!(current(serving, checkpoint), None);
    assert!(matches!(
        publish(serving, world.placement(), checkpoint, 0, Vec::new()),
        Err(AnchoredRecordPublicationFailure::EmptyPayload)
    ));

    let first = publish(serving, world.placement(), checkpoint, 0, value(1, 700)).unwrap();
    assert!(matches!(first, AnchoredRecordPublished::Published(_)));
    assert_eq!(first.version().generation(), 1);
    assert_eq!(current(serving, checkpoint), Some((1, value(1, 700))));

    let quiet = serving.physical_mutation_observation();
    let retry = publish(serving, world.placement(), checkpoint, 0, value(1, 700)).unwrap();
    assert_eq!(
        retry,
        AnchoredRecordPublished::AlreadyPublished(first.membership().clone())
    );
    assert!(matches!(
        publish(serving, world.placement(), checkpoint, 0, value(2, 700)),
        Err(AnchoredRecordPublicationFailure::GenerationConflict { current: 1 })
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );

    let second = publish(serving, world.placement(), checkpoint, 1, value(2, 900)).unwrap();
    assert!(matches!(second, AnchoredRecordPublished::Published(_)));
    assert_eq!(second.version().generation(), 2);
    assert_ne!(second.version().record(), first.version().record());
    assert_eq!(current(serving, checkpoint), Some((2, value(2, 900))));

    let quiet = serving.physical_mutation_observation();
    for (expected, payload) in [(0, value(1, 700)), (1, value(3, 10)), (5, value(3, 10))] {
        assert!(matches!(
            publish(serving, world.placement(), checkpoint, expected, payload),
            Err(AnchoredRecordPublicationFailure::GenerationConflict { current: 2 })
        ));
    }
    let records = serving.anchored_records().unwrap();
    assert!(matches!(
        records.read(checkpoint, 899),
        Err(AnchoredRecordReadFailure::LimitExceeded { payload_bytes: 900 })
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );
    assert_eq!(current(serving, checkpoint), Some((2, value(2, 900))));
}

#[test]
fn the_anchor_table_bounds_the_number_of_anchored_records() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-table")
        .expect("anchored record store");
    let names = ["anchor.one", "anchor.two", "anchor.three", "anchor.four"];
    for (index, text) in names.into_iter().enumerate() {
        publish(
            world.serving(),
            world.placement(),
            name(text),
            0,
            value(index as u8, 64),
        )
        .expect("an anchor slot is free");
    }
    assert!(matches!(
        publish(
            world.serving(),
            world.placement(),
            name("anchor.five"),
            0,
            value(9, 64)
        ),
        Err(AnchoredRecordPublicationFailure::AnchorTableFull)
    ));
    for (index, text) in names.into_iter().enumerate() {
        assert_eq!(
            current(world.serving(), name(text)),
            Some((1, value(index as u8, 64)))
        );
    }
}

#[test]
fn a_multi_megabyte_anchored_record_round_trips() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-large")
        .expect("anchored record store");
    let large = value(0x3c, 3 << 20);
    publish(
        world.serving(),
        world.placement(),
        name(CHECKPOINT),
        0,
        large.clone(),
    )
    .expect("a large publication");
    let (generation, bytes) = current(world.serving(), name(CHECKPOINT)).unwrap();
    assert_eq!(generation, 1);
    assert!(bytes == large, "the large value changed");
}

/// Interleaves journal entries with two anchored records, abandons the
/// runtime without closing it, and requires production recovery to serve the
/// latest durable generation of each and the whole journal, twice.
#[test]
fn anchored_records_and_the_journal_survive_an_abandoned_runtime() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-abandoned")
        .expect("anchored record store");
    let (checkpoint, home) = (name(CHECKPOINT), name(HOME));
    let serving = world.serving();
    publish_journal_entry(serving, world.placement(), 1);
    publish(serving, world.placement(), checkpoint, 0, value(1, 2_000)).unwrap();
    publish_journal_entry(serving, world.placement(), 2);
    publish(serving, world.placement(), home, 0, value(7, 120)).unwrap();
    publish_journal_entry(serving, world.placement(), 3);
    publish(serving, world.placement(), checkpoint, 1, value(2, 30_000)).unwrap();
    publish_journal_entry(serving, world.placement(), 4);
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "abandoned anchored records");
    assert_journal(&recovered, 4);
    assert_eq!(current(&recovered, checkpoint), Some((2, value(2, 30_000))));
    assert_eq!(current(&recovered, home), Some((1, value(7, 120))));
    publish(&recovered, placement(), checkpoint, 2, value(3, 5_000)).unwrap();
    publish_journal_entry(&recovered, placement(), 5);
    let _ = recovered.close();

    let again = serve(retained.path(), "second recovery");
    assert_journal(&again, 5);
    assert_eq!(current(&again, checkpoint), Some((3, value(3, 5_000))));
    assert_eq!(current(&again, home), Some((1, value(7, 120))));
    let _ = again.close();
}
