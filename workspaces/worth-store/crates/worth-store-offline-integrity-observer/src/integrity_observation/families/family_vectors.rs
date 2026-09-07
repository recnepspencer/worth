#![allow(dead_code)]
#[path = "../../../tests/phase_4_literal_vectors/checkpoint_records.rs"]
mod checkpoint_records;
#[path = "../../../tests/phase_4_literal_vectors/durable_frames.rs"]
mod durable_frames;
#[path = "../../../tests/phase_4_literal_vectors/oracle.rs"]
mod oracle;
#[path = "../../../tests/phase_4_literal_vectors/physical_work_obligation.rs"]
mod physical_work_obligation;
#[path = "../../../tests/phase_4_literal_vectors/wal_frame.rs"]
mod wal_frame;

use super::{
    bootstrap_catalog::read_bootstrap_catalog,
    extent::{read_extent_chunk, read_extent_manifest},
    free_space::{read_free_space_header, read_free_space_membership},
    root_routing::read_root_routing,
    segment_membership::read_segment_membership,
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters as Counters, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalDamageCause as Cause, OfflineUnsupportedVersionAxis,
};
use worth_foundational::PhysicalArtifactFamily as Family;

const FORMAT: [u8; 10] = [1, 0, 0, 64, 0, 0, 1, 1, 1, 24];
fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}
fn record(epoch: u8, ordinal: u64) -> [u8; 24] {
    let mut value = [epoch; 24];
    value[16..].copy_from_slice(&ordinal.to_le_bytes());
    value
}
fn segment_key(segment: u64, page: u64) -> Vec<u8> {
    [segment.to_le_bytes(), page.to_le_bytes()].concat()
}
fn free_key(class: u8, owner: u64) -> Vec<u8> {
    let mut key = vec![0; 16];
    key[0] = class;
    key[8..].copy_from_slice(&owner.to_le_bytes());
    key
}
fn expected(family: Family, generation: u64, scope: ChildScope) -> ChildExpectation {
    ChildExpectation {
        path: "literal".into(),
        family,
        generation,
        format: FORMAT,
        offset: 0,
        length: None,
        checksum: None,
        scope,
    }
}

#[test]
fn every_durable_family_reader_consumes_frozen_bytes_and_rejects_poison() {
    let root_record = record(0xa1, 5);
    let cases = [
        (
            durable_frames::ROOT_ROUTING,
            expected(
                Family::RootRoutingBlock,
                11,
                ChildScope::Tree {
                    tree: 71,
                    block: 3,
                    level: 0,
                    capacity: 2,
                    first: root_record.to_vec(),
                    last: root_record.to_vec(),
                },
            ),
        ),
        (
            durable_frames::SEGMENT_MEMBERSHIP,
            expected(
                Family::SegmentMembershipBlock,
                11,
                ChildScope::Tree {
                    tree: 73,
                    block: 5,
                    level: 0,
                    capacity: 2,
                    first: segment_key(13, 17),
                    last: segment_key(13, 17),
                },
            ),
        ),
        (
            durable_frames::FREE_SPACE_HEADER,
            expected(
                Family::FreeSpaceHeader,
                6,
                ChildScope::FreeSpace {
                    tree: 8,
                    capacity: 2,
                },
            ),
        ),
        (
            durable_frames::FREE_SPACE_MEMBERSHIP,
            expected(
                Family::FreeSpaceMembershipBlock,
                6,
                ChildScope::Tree {
                    tree: 8,
                    block: 1,
                    level: 0,
                    capacity: 2,
                    first: free_key(1, 7),
                    last: free_key(2, 5),
                },
            ),
        ),
        (
            durable_frames::EXTENT_MANIFEST,
            expected(
                Family::ExtentManifest,
                5,
                ChildScope::ExtentManifest {
                    extent: 4,
                    record: record(0x22, 7),
                    logical_bytes: 6,
                },
            ),
        ),
        (durable_frames::EXTENT_CHUNK, {
            let mut scope = expected(
                Family::ExtentChunkFrame,
                5,
                ChildScope::ExtentChunk {
                    extent: 4,
                    record: record(0x22, 7),
                    logical_bytes: 6,
                    logical_offset: 0,
                    ordinal: 1,
                },
            );
            scope.length = Some(118);
            scope
        }),
    ];
    for (literal, scope) in cases {
        let clean = hex(literal);
        assert!(
            inspect(&clean, &scope).is_ok(),
            "{:?}: {:?}",
            scope.family,
            inspect(&clean, &scope)
        );
        let mut poison = clean.clone();
        let last = poison.len() - 1;
        poison[last] ^= 1;
        assert!(
            matches!(inspect(&poison,&scope),Err(Outcome::Damaged(damage)) if damage.cause()==Cause::ChecksumMismatch),
            "{:?}",
            scope.family
        );
        let mut unsupported = clean.clone();
        unsupported[9] = 3;
        assert!(
            matches!(inspect(&unsupported,&scope),Err(Outcome::Unsupported(version)) if version.axis()==OfflineUnsupportedVersionAxis::EnvelopeSchema)
        );
        let mut wrong_scope = scope.clone();
        wrong_scope.generation += 1;
        assert!(
            matches!(inspect(&clean,&wrong_scope),Err(Outcome::Damaged(damage)) if damage.cause()==Cause::ScopeMismatch),
            "{:?}",
            scope.family
        );
        for prefix in [0, 1, 47, clean.len() - 1] {
            assert!(
                inspect(&clean[..prefix], &scope).is_err(),
                "{:?} prefix {prefix}",
                scope.family
            );
        }
    }
    let bootstrap = hex(durable_frames::BOOTSTRAP);
    assert_eq!(
        read_bootstrap_catalog(&bootstrap, [7; 16], &mut Counters::default()),
        Ok(())
    );
    assert!(
        matches!(read_bootstrap_catalog(&bootstrap,[8;16],&mut Counters::default()),Err(Outcome::Damaged(damage)) if damage.cause()==Cause::ScopeMismatch)
    );
}

fn inspect(bytes: &[u8], scope: &ChildExpectation) -> Result<Vec<ChildExpectation>, Outcome> {
    let counters = &mut Counters::default();
    match scope.family {
        Family::RootRoutingBlock => read_root_routing(bytes, scope, counters),
        Family::SegmentMembershipBlock => read_segment_membership(bytes, scope, counters),
        Family::FreeSpaceHeader => read_free_space_header(bytes, scope, counters),
        Family::FreeSpaceMembershipBlock => read_free_space_membership(bytes, scope, counters),
        Family::ExtentManifest => read_extent_manifest(bytes, scope, 100, counters),
        Family::ExtentChunkFrame => read_extent_chunk(bytes, scope, counters),
        _ => unreachable!(),
    }
}

#[test]
fn independent_wal_and_pending_readers_preserve_scope_and_version() {
    let wal = hex(wal_frame::FRAME_HEX);
    let report = super::wal::read_wal_segment(&wal, 1, 2, 10, &mut Counters::default());
    assert_eq!(report.len(), 1);
    assert_eq!(report[0].outcome, Outcome::Intact);
    assert_eq!(report[0].length, 151);
    let wrong = super::wal::read_wal_segment(&wal, 2, 2, 10, &mut Counters::default());
    assert!(
        matches!(&wrong[0].outcome,Outcome::Damaged(damage) if damage.cause()==Cause::ScopeMismatch)
    );
    let mut poisoned = wal.clone();
    poisoned[116] ^= 1;
    let report = super::wal::read_wal_segment(&poisoned, 1, 2, 10, &mut Counters::default());
    assert!(
        matches!(&report[0].outcome,Outcome::Damaged(damage) if damage.cause()==Cause::ChecksumMismatch)
    );
    let pending = hex(physical_work_obligation::OPERATION_3_HEX);
    let store = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    assert_eq!(
        super::physical_work::read_physical_work(
            &pending,
            store,
            (1, 2, 3),
            &mut Counters::default()
        ),
        Ok(())
    );
    assert!(
        matches!(super::physical_work::read_physical_work(&pending,store,(1,2,4),&mut Counters::default()),Err(Outcome::Damaged(damage)) if damage.cause()==Cause::ScopeMismatch)
    );
}

#[test]
fn checkpoint_stream_preserves_record_kinds_and_selective_aggregates() {
    let header = hex(checkpoint_records::HEADER);
    let dirty = hex(checkpoint_records::DIRTY_BASIS);
    let compaction = hex(checkpoint_records::BINDING_COMPACTION);
    let binding = hex("57435037524543000104000003000000aabbcc9133843e");
    let footer = hex(checkpoint_records::FOOTER);
    let stream = [header, dirty, compaction, binding, footer].concat();
    let store = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let records =
        super::checkpoint::read_checkpoint(&stream, store, None, 100, &mut Counters::default());
    assert_eq!(records.len(), 5);
    assert!(
        records
            .iter()
            .all(|record| record.outcome == Outcome::Intact),
        "{:?}",
        records
            .iter()
            .map(|record| &record.outcome)
            .collect::<Vec<_>>()
    );
    for (index, offset) in [0, 164, 232, 268, 291].into_iter().enumerate() {
        let mut unsupported = stream.clone();
        unsupported[offset + 8] = 2;
        let records = super::checkpoint::read_checkpoint(
            &unsupported,
            store,
            None,
            100,
            &mut Counters::default(),
        );
        assert_eq!(records.len(), index + 1);
        assert!(
            matches!(&records[index].outcome,Outcome::Unsupported(version) if version.axis()==OfflineUnsupportedVersionAxis::CheckpointRecord)
        );
    }
    let mut aggregate = stream.clone();
    aggregate[164 + 16 + 40] ^= 1;
    let crc = crate::integrity_observation::crc32c::crc32c(&[&aggregate[164..228]]);
    aggregate[228..232].copy_from_slice(&crc.to_le_bytes());
    let records =
        super::checkpoint::read_checkpoint(&aggregate, store, None, 100, &mut Counters::default());
    assert_eq!(records.len(), 5);
    assert!(
        matches!(&records[4].outcome,Outcome::Damaged(damage) if damage.cause()==Cause::ChecksumMismatch)
    );
}
