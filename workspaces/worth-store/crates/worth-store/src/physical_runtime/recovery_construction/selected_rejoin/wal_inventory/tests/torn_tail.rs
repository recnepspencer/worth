//! Only the newest WAL segment may end in a torn frame: C.8 keeps its valid
//! prefix, while a truncated frame in an older segment is a WAL fate failure.
use super::*;
use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};

/// One intact frame, optionally followed by a frame missing its last byte.
fn segment(root: &std::path::Path, segment: u64, lsn: u64, torn: bool) -> Vec<u8> {
    let identity = worth_store_physical_format::WalSegmentIdentity::new(segment, 1).unwrap();
    let frame = |lsn| {
        encode_wal_frame_v1(
            WalFrameV1EncodeRequest::from_segment_identity(
                identity,
                lsn,
                lsn + 1,
                b"torn-tail-owner",
                b"payload",
            )
            .unwrap(),
        )
    };
    let mut bytes = frame(lsn);
    if torn {
        let next = frame(lsn + 2);
        bytes.extend_from_slice(&next[..next.len() - 1]);
    }
    let directory = root.join("families/wal");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join(format!("segment-{segment}-generation-1.wal")),
        &bytes,
    )
    .unwrap();
    bytes
}

#[test]
fn truncated_frame_is_residue_only_on_the_newest_segment() {
    let (root, media, coordination) = fixture::coordination();
    segment(root.path(), 1, 2, true);
    segment(root.path(), 2, 10, false);
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    assert!(matches!(
        admit_complete_inventory(&mut discovery, &coordination),
        Err(Denial::WalFate)
    ));
    let media = discovery.finish();
    drop(coordination);
    drop(media);
    drop(root);

    let (root, media, coordination) = fixture::coordination();
    let older = segment(root.path(), 1, 2, false);
    let newest = segment(root.path(), 2, 10, true);
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let inventory = admit_complete_inventory(&mut discovery, &coordination).unwrap();
    assert_eq!(
        inventory
            .frames()
            .iter()
            .map(|frame| frame.lsn_start())
            .collect::<Vec<_>>(),
        [2, 10],
        "the valid prefix of each segment is admitted; the torn suffix is not"
    );
    assert_eq!(
        inventory
            .artifacts
            .iter()
            .map(|entry| entry.length)
            .collect::<Vec<_>>(),
        [older.len() as u64, newest.len() as u64],
        "the fingerprint still binds the torn segment's exact bytes"
    );
    drop(inventory);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}
