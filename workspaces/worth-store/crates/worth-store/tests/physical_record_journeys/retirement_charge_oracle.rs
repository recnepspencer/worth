use std::collections::BTreeSet;
use std::path::Path;

use sha2::{Digest, Sha256};

const HEADER: usize = 116;
const FOOTER: usize = 32;
const BINDING_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";
const CANONICAL_REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const REWRITE_REDO_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v2";
const COPY_PUBLICATION_DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";

/// The durable end of a checked, independently framed WAL inventory.
pub(crate) struct WalByteSnapshot {
    frames: Vec<WalFrameInventoryEntry>,
}

#[derive(Debug, PartialEq, Eq)]
struct WalFrameInventoryEntry {
    artifact: std::path::PathBuf,
    start: u64,
    end: u64,
    encoded_bytes: u64,
    digest: [u8; 32],
    published_successor: Option<u64>,
}

pub(crate) fn wal_bytes(root: &Path) -> WalByteSnapshot {
    let frames = framed_wal(root);
    assert!(!frames.is_empty(), "retirement fixture has WAL");
    WalByteSnapshot { frames }
}

pub(crate) fn assert_one_page_released_net_of_wal(
    root: &Path,
    before_wal: WalByteSnapshot,
    before_charge: u64,
    after_charge: u64,
    page_bytes: u64,
) {
    // LSNs identify both newly appended frames and a reclaimed prefix when
    // retirement completion releases the WAL segment held by its intent.
    let after_wal = framed_wal(root);
    let last_lsn_end = before_wal.frames.last().unwrap().end;
    let old_frame_count = after_wal.partition_point(|frame| frame.start < last_lsn_end);
    let retained_old = &after_wal[..old_frame_count];
    let pruned_count = before_wal
        .frames
        .len()
        .checked_sub(old_frame_count)
        .expect("retirement cannot add frames before the prior WAL frontier");
    assert_eq!(
        retained_old,
        &before_wal.frames[pruned_count..],
        "retirement may reclaim only a complete WAL prefix, not alter retained frames"
    );
    let pruned_wal: u64 = before_wal.frames[..pruned_count]
        .iter()
        .map(|frame| frame.encoded_bytes)
        .sum();
    let pruned_publications = before_wal.frames[..pruned_count]
        .iter()
        .filter_map(|frame| frame.published_successor)
        .collect::<BTreeSet<_>>();
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let pruned_metadata: u64 = pruned_publications
        .iter()
        .map(|generation| {
            crate::publication_metadata_oracle::actual_publication_metadata_from_media(
                root,
                *generation,
                format,
            )
        })
        .sum();
    let new_frames = &after_wal[old_frame_count..];
    assert_eq!(
        new_frames.first().map(|frame| frame.start),
        Some(last_lsn_end),
        "retirement completion must append at the prior WAL frontier"
    );
    let wal_growth: u64 = new_frames.iter().map(|frame| frame.encoded_bytes).sum();
    assert_eq!(
        i128::from(before_charge) - i128::from(after_charge) + i128::from(wal_growth)
            - i128::from(pruned_wal)
            - i128::from(pruned_metadata),
        i128::from(page_bytes),
        "exact charge conservation after {wal_growth} new WAL bytes, {pruned_wal} pruned WAL bytes, and {pruned_metadata} pruned publication metadata bytes"
    );
}

fn framed_wal(root: &Path) -> Vec<WalFrameInventoryEntry> {
    let mut frames = Vec::new();
    for entry in std::fs::read_dir(root.join("families/wal")).unwrap() {
        let artifact = entry.unwrap().path();
        let bytes = std::fs::read(&artifact).unwrap();
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
            frames.push(WalFrameInventoryEntry {
                artifact: artifact.clone(),
                start,
                end,
                encoded_bytes: u64::try_from(frame_end - offset).unwrap(),
                digest: Sha256::digest(&bytes[offset..frame_end]).into(),
                published_successor: published_successor(&bytes[offset + HEADER..payload_end]),
            });
            offset = frame_end;
        }
    }
    frames.sort_unstable_by_key(|frame| frame.start);
    assert!(frames.windows(2).all(|pair| pair[0].end == pair[1].start));
    frames
}

fn published_successor(payload: &[u8]) -> Option<u64> {
    let (first, rest) = field(payload).expect("checked WAL payload has its first field");
    if first.starts_with(b"store.") {
        // Retirement and maintenance records are not root publications.
        return None;
    }
    let (binding_domain, _) = field(first).expect("publication member has binding domain");
    assert_eq!(binding_domain, BINDING_DOMAIN);
    let (redo, remainder) = field(rest).expect("publication member has canonical redo");
    assert!(remainder.is_empty());
    let (domain, body) = field(redo).expect("publication redo has domain");
    let source = if domain == CANONICAL_REDO_DOMAIN {
        canonical_source_root(body)
    } else if domain == REWRITE_REDO_DOMAIN {
        assert_eq!(body.len(), 280);
        let source = u64_at(body, 64);
        assert_eq!(u64_at(body, 208), source.checked_add(1).unwrap());
        source
    } else if domain == COPY_PUBLICATION_DOMAIN {
        let (_, body) = exact(body, 8);
        let (projection, remainder) = field(body).expect("copy publication has projection");
        assert!(remainder.is_empty());
        projection_source_root(projection)
    } else {
        panic!("unmapped authenticated WAL publication domain: {domain:?}")
    };
    Some(
        source
            .checked_add(1)
            .expect("publication successor generation"),
    )
}

fn canonical_source_root(mut body: &[u8]) -> u64 {
    let (count, rest) = take_u64(body);
    assert!(count > 0 && count <= rest.len() as u64);
    body = rest;
    for _ in 0..count {
        let (_, rest) = exact(body, 4 + 8);
        let (targets, mut rest) = take_u64(rest);
        assert!(targets > 0 && targets <= rest.len() as u64 / 40);
        for _ in 0..targets {
            let (_, tail) = field(rest).expect("canonical target claim");
            (_, rest) = exact(tail, 32);
        }
        (_, body) = field(rest).expect("canonical record");
    }
    let (projection, remainder) = field(body).expect("canonical projection");
    assert!(remainder.is_empty());
    projection_source_root(projection)
}

fn projection_source_root(projection: &[u8]) -> u64 {
    let (domain, body) = field(projection).expect("projection domain");
    assert_eq!(domain, b"store.physical.recovery-projection.v16");
    let (source, _) = take_u64(body);
    assert_ne!(source, 0);
    source
}

fn field(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let length = usize::try_from(u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?)).ok()?;
    let end = 8_usize.checked_add(length)?;
    Some((bytes.get(8..end)?, bytes.get(end..)?))
}

fn exact(bytes: &[u8], length: usize) -> (&[u8], &[u8]) {
    (bytes.get(..length).unwrap(), bytes.get(length..).unwrap())
}

fn take_u64(bytes: &[u8]) -> (u64, &[u8]) {
    let (value, rest) = exact(bytes, 8);
    (u64::from_le_bytes(value.try_into().unwrap()), rest)
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
