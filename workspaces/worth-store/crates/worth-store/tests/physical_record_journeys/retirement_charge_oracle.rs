use std::path::Path;

use sha2::{Digest, Sha256};

const HEADER: usize = 116;
const FOOTER: usize = 32;

/// The durable end of a checked, independently framed WAL inventory.
pub(crate) struct WalByteSnapshot {
    last_lsn_end: u64,
}

pub(crate) fn wal_bytes(root: &Path) -> WalByteSnapshot {
    let frames = framed_wal(root);
    WalByteSnapshot {
        last_lsn_end: frames.last().expect("retirement fixture has WAL").1,
    }
}

pub(crate) fn assert_one_page_released_net_of_wal(
    root: &Path,
    before_wal: WalByteSnapshot,
    before_charge: u64,
    after_charge: u64,
    page_bytes: u64,
) {
    // LSNs identify newly appended frames even when the file set changes.
    // These fixtures expect no other charged release during this transition.
    let new_frames = framed_wal(root)
        .into_iter()
        .filter(|(start, _, _)| *start >= before_wal.last_lsn_end)
        .collect::<Vec<_>>();
    assert_eq!(
        new_frames.first().map(|frame| frame.0),
        Some(before_wal.last_lsn_end),
        "retirement completion must append at the prior WAL frontier"
    );
    let wal_growth: u64 = new_frames.iter().map(|frame| frame.2).sum();
    assert_eq!(before_charge - after_charge + wal_growth, page_bytes);
}

fn framed_wal(root: &Path) -> Vec<(u64, u64, u64)> {
    let mut frames = Vec::new();
    for entry in std::fs::read_dir(root.join("families/wal")).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap();
        let mut offset = 0;
        while offset < bytes.len() {
            let header = &bytes[offset..offset + HEADER];
            assert_eq!(&header[..8], b"WORTHWAL");
            assert_eq!(&header[8..10], &1_u16.to_le_bytes());
            assert_eq!(&header[10..12], &(HEADER as u16).to_le_bytes());
            let start = u64::from_le_bytes(header[28..36].try_into().unwrap());
            let end = u64::from_le_bytes(header[36..44].try_into().unwrap());
            assert!(start < end);
            let payload_len =
                usize::try_from(u64::from_le_bytes(header[44..52].try_into().unwrap())).unwrap();
            let payload_end = offset + HEADER + payload_len;
            let frame_end = payload_end + FOOTER;
            assert_eq!(
                Sha256::digest(&bytes[offset + HEADER..payload_end])[..],
                header[84..116]
            );
            assert_eq!(
                Sha256::digest(&bytes[offset..payload_end])[..],
                bytes[payload_end..frame_end]
            );
            frames.push((start, end, u64::try_from(frame_end - offset).unwrap()));
            offset = frame_end;
        }
    }
    frames.sort_unstable_by_key(|frame| frame.0);
    assert!(frames.windows(2).all(|pair| pair[0].1 == pair[1].0));
    frames
}
