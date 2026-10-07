#![allow(dead_code)]
#[path = "../../../../tests/phase_4_literal_vectors/checkpoint_records.rs"]
mod checkpoint_records;
#[path = "../../../../tests/support/fixtures.rs"]
mod fixtures;
#[path = "../../../../tests/phase_4_literal_vectors/oracle.rs"]
mod oracle;
#[path = "../../../../tests/support/temporary_root.rs"]
mod temporary_root;

use super::*;
use crate::integrity_observation::{
    journal_walk::observe_journals, retirement_evidence::RetirementEvidence,
    root_protocol_walk::observe_root_protocol, OfflineIntegrityObservationLimits,
};
use std::time::Instant;
use temporary_root::TemporaryRoot;

const STORE: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

fn checkpoint() -> Vec<u8> {
    let mut bytes = [
        checkpoint_records::HEADER,
        checkpoint_records::DIRTY_BASIS,
        checkpoint_records::BINDING_COMPACTION,
        checkpoint_records::BINDING,
        checkpoint_records::FOOTER,
    ]
    .into_iter()
    .flat_map(oracle::decode_hex)
    .collect::<Vec<_>>();
    // Bind the literal checkpoint to the independently encoded root fixture.
    bytes[56..64].copy_from_slice(&1_u64.to_le_bytes());
    bytes[64..72].copy_from_slice(&7_u64.to_le_bytes());
    reseal_header(&mut bytes);
    bytes
}

fn reseal_header(bytes: &mut [u8]) {
    let checksum = crate::integrity_observation::crc32c::crc32c(&[&bytes[..160]]);
    bytes[160..164].copy_from_slice(&checksum.to_le_bytes());
}

fn observe(
    bytes: &[u8],
    staged: bool,
) -> crate::integrity_observation::journal_walk::ObservedJournals {
    let fixture = fixtures::clean_store("expiry-checkpoint-evidence");
    for directory in ["staging", "families/wal", "families/physical-work"] {
        std::fs::create_dir_all(fixture.store.join(directory)).unwrap();
    }
    let path = if staged {
        "staging/checkpoint-0000000000000007.candidate"
    } else {
        "families/checkpoint.current"
    };
    std::fs::write(fixture.store.join(path), bytes).unwrap();
    let root = std::fs::canonicalize(&fixture.store).unwrap();
    let limits =
        OfflineIntegrityObservationLimits::new(200, 1 << 20, 8, 8, 0, 30_000, 1 << 20).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    let roots = observe_root_protocol(&root, Some(STORE), &mut walk).unwrap();
    let selected = roots
        .roots
        .iter()
        .find(|root| Some(root.generation) == roots.current_generation);
    assert!(
        selected.is_some(),
        "root fixture must establish real selector authority"
    );
    observe_journals(
        &root,
        Some(STORE),
        &mut walk,
        &mut RetirementEvidence::default(),
        selected,
    )
}

#[test]
fn journal_requires_complete_current_checkpoint_not_header_or_candidate() {
    let bytes = checkpoint();
    assert!(matches!(
        observe(&bytes, false).checkpoint,
        SelectedCheckpointEvidence::Validated { sequence: 7, .. }
    ));
    let staged = observe(&bytes, true);
    assert!(matches!(
        staged.checkpoint,
        SelectedCheckpointEvidence::Absent
    ));
    assert!(staged
        .artifacts
        .iter()
        .all(|row| row.outcome() == &Outcome::Intact));
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    for invalid in [&bytes[..164], &bytes[..bytes.len() - 1], corrupt.as_slice()] {
        let result = observe(invalid, false);
        assert!(matches!(
            result.checkpoint,
            SelectedCheckpointEvidence::Unavailable(_)
        ));
        assert!(result
            .artifacts
            .iter()
            .any(|row| row.outcome() != &Outcome::Intact));
    }
}

#[test]
fn journal_rejects_resealed_checkpoint_source_root_and_tree_mismatch() {
    for (offset, value) in [(56, 2_u64), (64, 8_u64)] {
        let mut bytes = checkpoint();
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        reseal_header(&mut bytes);
        let result = observe(&bytes, false);
        assert!(matches!(
            result.checkpoint,
            SelectedCheckpointEvidence::Unavailable(Outcome::Damaged(_))
        ));
        assert!(result
            .artifacts
            .iter()
            .any(|row| matches!(row.outcome(), Outcome::Damaged(_))));
    }
}

fn schema_three_frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = b"WCP7REC\0".to_vec();
    bytes.extend([3, kind, 0, 0]);
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend(payload);
    let checksum = crate::integrity_observation::crc32c::crc32c(&[&bytes]);
    bytes.extend(checksum.to_le_bytes());
    bytes
}

fn schema_three_no_release_checkpoint(root_sha: [u8; 32]) -> Vec<u8> {
    let legacy = checkpoint();
    let mut source_offset = 0;
    let mut bytes = Vec::new();
    let mut dirty = Vec::new();
    let mut bindings = Vec::new();
    for (kind, length) in [(1, 164), (2, 68), (3, 36), (4, 23)] {
        let record = schema_three_frame(
            kind,
            &legacy[source_offset + 16..source_offset + length - 4],
        );
        if kind == 2 {
            dirty = record.clone();
        }
        if kind == 4 {
            bindings = record.clone();
        }
        bytes.extend(record);
        source_offset += length;
    }
    let domain = b"store.physical.checkpoint.released-drop-custody.v1";
    let mut marker = Vec::new();
    marker.extend((domain.len() as u64).to_le_bytes());
    marker.extend(domain);
    marker.extend([1, 3]);
    marker.extend(STORE);
    marker.extend(7_u64.to_le_bytes());
    marker.extend(1_u64.to_le_bytes());
    marker.extend(root_sha);
    marker.extend([0; 72]);
    let certificate = schema_three_frame(7, &marker);
    bytes.extend(&certificate);
    let mut footer = [0; 184];
    footer[..136].copy_from_slice(&legacy[source_offset + 16..source_offset + 152]);
    footer[32..64].copy_from_slice(&crate::integrity_observation::sha256::sha256(&dirty));
    footer[104..136].copy_from_slice(&crate::integrity_observation::sha256::sha256(&bindings));
    footer[136..144].copy_from_slice(&1_u64.to_le_bytes());
    footer[144..152].copy_from_slice(&(certificate.len() as u64).to_le_bytes());
    footer[152..184].copy_from_slice(&crate::integrity_observation::sha256::sha256(&certificate));
    bytes.extend(schema_three_frame(5, &footer));
    bytes
}

fn observe_schema_three_no_release(wrong_root: bool) -> SelectedCheckpointEvidence {
    let fixture = fixtures::clean_store("no-release-root-binding");
    for directory in ["staging", "families/wal", "families/physical-work"] {
        std::fs::create_dir_all(fixture.store.join(directory)).unwrap();
    }
    let root_bytes = std::fs::read(
        fixture
            .store
            .join("families/records/roots/root-0000000000000001.manifest"),
    )
    .unwrap();
    let source_root_sha = crate::integrity_observation::sha256::sha256(&root_bytes);
    let claimed_sha = if wrong_root { [9; 32] } else { source_root_sha };
    std::fs::write(
        fixture.store.join("families/checkpoint.current"),
        schema_three_no_release_checkpoint(claimed_sha),
    )
    .unwrap();
    let root = std::fs::canonicalize(&fixture.store).unwrap();
    let limits =
        OfflineIntegrityObservationLimits::new(200, 1 << 20, 8, 8, 0, 30_000, 1 << 20).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    let roots = observe_root_protocol(&root, Some(STORE), &mut walk).unwrap();
    let selected = roots
        .roots
        .iter()
        .find(|candidate| Some(candidate.generation) == roots.current_generation);
    observe_journals(
        &root,
        Some(STORE),
        &mut walk,
        &mut RetirementEvidence::default(),
        selected,
    )
    .checkpoint
}

#[test]
fn no_release_claim_binds_exact_acquired_source_root_frame() {
    assert!(matches!(
        observe_schema_three_no_release(false),
        SelectedCheckpointEvidence::Validated {
            release_claim: Some(_),
            ..
        }
    ));
    let SelectedCheckpointEvidence::Unavailable(Outcome::Damaged(damage)) =
        observe_schema_three_no_release(true)
    else {
        panic!("wrong source root digest must be unavailable");
    };
    assert_eq!(damage.cause(), Cause::ScopeMismatch);
}
