//! A resume frontier of the released session against one of another owner.
//! Both name the same reachable chunk of the released session.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobRecordKind, BlobSessionFrontierV1, PersistedRecordIdentity, PhysicalTierClass,
    SelectedRecordContentClass,
};

use super::super::inventory::{SelectedBlobFact, SelectedReleaseFact, SelectedReleaseInventory};
use super::ExternalEdgeAudit;
use crate::physical_runtime::terminal_head_retirement_fixture::basis;

const CHUNK: u64 = 20;
const FRONTIER: u64 = 21;
const OTHER_SESSION: [u8; 16] = [0x55; 16];

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn fact(
    ordinal: u64,
    kind: BlobRecordKind,
    frame_sha256: [u8; 32],
    blob: SelectedBlobFact,
    reachable: bool,
) -> SelectedReleaseFact {
    SelectedReleaseFact {
        record: record(ordinal),
        class: SelectedRecordContentClass::Blob(kind),
        tier: PhysicalTierClass::Primary,
        payload_bytes: 0,
        frame_sha256,
        blob,
        reachable,
        protected: false,
    }
}

struct Folded {
    chunk_protected: bool,
    publication_referenced: bool,
    digest: [u8; 32],
    expected_digest: [u8; 32],
}

/// Folds one selected frontier, written by `writer`, that names the one
/// reachable chunk of the released session.
fn fold_frontier_of(writer: [u8; 16]) -> Folded {
    let basis = basis();
    let store = basis.publication().store();
    let frame = BlobSessionFrontierV1::new(
        store,
        writer,
        record(19),
        [8; 32],
        64,
        64 << 16,
        record(CHUNK),
        [9; 32],
    )
    .unwrap()
    .encode();
    let frame_sha256: [u8; 32] = Sha256::digest(&frame).into();
    let mut inventory = SelectedReleaseInventory {
        basis,
        publication_selected: false,
        facts: vec![
            fact(
                CHUNK,
                BlobRecordKind::Chunk,
                [2; 32],
                SelectedBlobFact::Chunk {
                    store,
                    session: basis.session(),
                    ordinal: 63,
                    content_digest: [9; 32],
                    covered_bytes: 1 << 16,
                    chunk_size: 1 << 16,
                },
                true,
            ),
            fact(
                FRONTIER,
                BlobRecordKind::SessionFrontier,
                frame_sha256,
                SelectedBlobFact::Frontier { session: writer },
                false,
            ),
        ],
        manifests: Vec::new(),
        descriptors: Vec::new(),
        reservations: Vec::new(),
        occupied_attempts: Vec::new(),
        selected_route_inventory_sha256: [0; 32],
    };
    let mut audit = ExternalEdgeAudit::new(&mut inventory);
    audit.source(record(FRONTIER), &frame).unwrap();
    let (publication_referenced, digest) = audit.finish();

    // One source row and one frontier edge to a reachable same-session chunk.
    let mut expected = Sha256::new();
    expected.update(b"store.physical.released-drop-external-edges.v1");
    expected.update([0]);
    expected.update([1; 16]);
    expected.update(FRONTIER.to_le_bytes());
    expected.update(frame_sha256);
    expected.update([1, 9]);
    expected.update([1; 16]);
    expected.update(FRONTIER.to_le_bytes());
    expected.update([1; 16]);
    expected.update(CHUNK.to_le_bytes());
    expected.update([0, 1, 1]);
    expected.update(1_u64.to_le_bytes());
    expected.update(1_u64.to_le_bytes());
    Folded {
        chunk_protected: inventory.fact(record(CHUNK)).unwrap().protected,
        publication_referenced,
        digest,
        expected_digest: expected.finalize().into(),
    }
}

#[test]
fn a_frontier_of_the_released_session_is_audited_and_protects_nothing() {
    let own = fold_frontier_of(basis().session());
    assert!(
        !own.chunk_protected,
        "the released session's own frontier kept its chunk selected"
    );
    assert!(!own.publication_referenced);
    assert_eq!(own.digest, own.expected_digest);
}

#[test]
fn a_frontier_of_another_session_protects_the_chunk_it_names() {
    assert_ne!(basis().session(), OTHER_SESSION);
    let other = fold_frontier_of(OTHER_SESSION);
    assert!(
        other.chunk_protected,
        "another session's frontier lost the chunk it names"
    );
    assert!(!other.publication_referenced);
    assert_eq!(other.digest, other.expected_digest);
}
