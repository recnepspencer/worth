//! Arch 10, 21: persisted bytes regain an identity only by re-verifying a
//! birth. Tampering any typed birth part or the stored digest, an unsupported
//! format version, or non-canonical structure refuses typed before any
//! derived state is installed.

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    encode_commit_frame, rebuild_name_index_from_durable_log, PersistentNameDurabilityError,
    PersistentNameIdentity, PersistentNameWireSection, PERSISTENT_NAME_DURABLE_FORMAT_VERSION,
};
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use super::name_durable_world::{durable_naming_history, DurableNamingHistory};

/// Byte offsets of one persisted birth record's fields in the log.
struct BirthLayout {
    identity: usize,
    scheme: usize,
    occurrence: usize,
    role: usize,
    parents: usize,
    discriminator: usize,
}

fn read_len(bytes: &[u8], at: usize) -> usize {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

/// A name's first occurrence in the log is its birth record: nothing can
/// reference an identity before the frame that births it.
fn birth_layout(bytes: &[u8], name: PersistentNameIdentity) -> BirthLayout {
    let identity = bytes
        .windows(32)
        .position(|window| window == name.as_bytes())
        .expect("the log persists the birth");
    let scheme = identity + 32 + 1;
    let occurrence = scheme + 4 + read_len(bytes, scheme);
    let role = occurrence + 4 + read_len(bytes, occurrence);
    let parents = role + 4 + read_len(bytes, role);
    let discriminator = parents + 4 + 32 * read_len(bytes, parents);
    BirthLayout {
        identity,
        scheme,
        occurrence,
        role,
        parents,
        discriminator,
    }
}

fn full_log(history: &DurableNamingHistory) -> Vec<u8> {
    history.persisted_through(history.commits.len())
}

fn refusal_after(bytes: &[u8], at: usize, flip: u8) -> PersistentNameDurabilityError {
    let mut tampered = bytes.to_vec();
    tampered[at] ^= flip;
    rebuild_name_index_from_durable_log(&tampered).expect_err("tampered log refuses")
}

#[test]
fn every_tampered_birth_part_and_stored_digest_refuses_corrupt_birth_digest() {
    let history = durable_naming_history();
    let bytes = full_log(&history);
    let child = birth_layout(&bytes, history.split_child);
    assert_eq!(
        read_len(&bytes, child.parents),
        1,
        "a split child has one parent"
    );
    let last = |field: usize| field + 4 + read_len(&bytes, field) - 1;
    let tampers = [
        ("scheme_id", last(child.scheme)),
        ("occurrence_key", last(child.occurrence)),
        ("role_path", last(child.role)),
        ("parent_identities", child.parents + 4 + 31),
        ("canonical_discriminator", last(child.discriminator)),
        ("identity", child.identity),
    ];
    for (field, at) in tampers {
        match refusal_after(&bytes, at, 0x01) {
            PersistentNameDurabilityError::CorruptBirthDigest { stored, recomputed } => {
                assert_ne!(stored, recomputed, "{field}");
                let expected_stored = if field == "identity" {
                    let mut tampered = *history.split_child.as_bytes();
                    tampered[0] ^= 0x01;
                    tampered
                } else {
                    *history.split_child.as_bytes()
                };
                assert_eq!(stored, expected_stored, "{field}");
            }
            other => panic!("{field} tamper refused as {other:?}"),
        }
    }
}

#[test]
fn unsorted_duplicate_parents_and_malformed_discriminators_refuse_typed() {
    let history = durable_naming_history();
    let bytes = full_log(&history);
    let merged = birth_layout(&bytes, history.merged);
    assert_eq!(
        read_len(&bytes, merged.parents),
        2,
        "a minted merge has two parents"
    );
    let first = merged.parents + 4;
    let mut swapped = bytes.clone();
    swapped[first..first + 64].rotate_left(32);
    let mut duplicated = bytes.clone();
    duplicated.copy_within(first..first + 32, first + 32);
    for tampered in [swapped, duplicated] {
        assert_eq!(
            rebuild_name_index_from_durable_log(&tampered),
            Err(PersistentNameDurabilityError::NonCanonicalBirthParents {
                identity: *history.merged.as_bytes(),
            })
        );
    }
    let child = birth_layout(&bytes, history.split_child);
    assert_eq!(
        refusal_after(&bytes, child.discriminator + 4, 0x01),
        PersistentNameDurabilityError::MalformedBirthDiscriminator {
            identity: *history.split_child.as_bytes(),
        }
    );
}

#[test]
fn unsupported_format_versions_refuse_before_any_state() {
    let history = durable_naming_history();
    let bytes = full_log(&history);
    let header_version = 4;
    let first_frame_version = history.header().len() + 4;
    assert_eq!(
        bytes[header_version],
        PERSISTENT_NAME_DURABLE_FORMAT_VERSION
    );
    assert_eq!(
        bytes[first_frame_version],
        PERSISTENT_NAME_DURABLE_FORMAT_VERSION
    );
    for at in [header_version, first_frame_version] {
        for found in [0u8, 2] {
            let mut tampered = bytes.clone();
            tampered[at] = found;
            assert_eq!(
                rebuild_name_index_from_durable_log(&tampered),
                Err(PersistentNameDurabilityError::UnsupportedFormatVersion {
                    found,
                    supported: 1,
                })
            );
        }
    }
}

#[test]
fn non_canonical_structure_refuses_typed() {
    let history = durable_naming_history();
    let bytes = full_log(&history);
    let header = history.header();
    assert_eq!(
        refusal_after(&bytes, 0, 0x01),
        PersistentNameDurabilityError::UnrecognizedMagic {
            section: PersistentNameWireSection::LogHeader,
        }
    );
    assert_eq!(
        refusal_after(&bytes, header.len() - 1, 0x80),
        PersistentNameDurabilityError::MalformedUtf8 { field: "branch id" }
    );

    let (anchors, _) = &history.commits[0];
    let frame = encode_commit_frame(anchors, None).unwrap();
    let mut padded = header.clone();
    padded.extend_from_slice(&frame[..5]);
    padded.extend_from_slice(&((frame.len() - 9 + 1) as u32).to_be_bytes());
    padded.extend_from_slice(&frame[9..]);
    padded.push(0);
    assert_eq!(
        rebuild_name_index_from_durable_log(&padded),
        Err(PersistentNameDurabilityError::TrailingBytes { remaining: 1 })
    );

    let event = &anchors.lineage_events_for_test()[0];
    let kind = 9
        + 8
        + 1
        + 4
        + 8
        + 8
        + 8
        + 4
        + event.commit().branch_id.0.len()
        + 4
        + 8 * event.commit().parents.len()
        + 4
        + event.branch_id().0.len();
    let mut bad_kind = header.clone();
    bad_kind.extend_from_slice(&frame);
    bad_kind[header.len() + kind] = 0;
    assert_eq!(
        rebuild_name_index_from_durable_log(&bad_kind),
        Err(PersistentNameDurabilityError::InvalidTag {
            field: "event kind",
            tag: 0,
        })
    );

    let floor_above_commit = encode_commit_frame(anchors, Some(CommitId(anchors.commit().0 + 1)));
    let mut bytes = header.clone();
    bytes.extend_from_slice(&floor_above_commit.unwrap());
    assert_eq!(
        rebuild_name_index_from_durable_log(&bytes),
        Err(PersistentNameDurabilityError::NonCanonical {
            field: "retention floor",
        })
    );

    let mut reversed = history.commits[1].0.clone();
    reversed.lineage_events_mut_for_test().reverse();
    assert_eq!(
        encode_commit_frame(&reversed, None),
        Err(PersistentNameDurabilityError::NonCanonical {
            field: "lineage event order",
        })
    );
}
