//! The first recovery publishes a V3 drop without folding its WAL into a
//! checkpoint. A second, independent C8 process must preserve that result.

pub(crate) use {std::fs, std::path::Path, std::process::Command, std::process::Output, std::time::Duration};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store_physical_format::BlobReclaimDescriptorV3, worth_store_physical_format::BlobRecordKind};

pub(crate) use {crate::physical_blob_journeys::blob_crash::kill_at, crate::blob_crash::recovery::recover_closed_store, crate::blob_reclaim_released_crash::retirement_witness, crate::physical_blob_journeys::blob_reclaim_released_crash::ROLE};

pub(crate) const C5_HEADER: usize = 48;
pub(crate) const C5_EXTENT_METADATA: usize = 64;
pub(crate) const C11_HEADER: usize = 48;

pub(crate) fn run_recovery(root: &Path) -> Output {
    let binary = worth_store_test_support::FRESH_PROCESS_RECOVERY.path();
    std::process::Command::new(binary)
        .arg(root)
        .arg("--bounded-profile=c11-blob-crash-v1")
        .output()
        .unwrap()
}

pub(crate) fn mutate_selected_descriptor_custody(root: &Path) {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = std::fs::read(&path).unwrap();
        for inner in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer) = inner.checked_sub(C5_HEADER + C5_EXTENT_METADATA) else {
                continue;
            };
            if !media[inner..].starts_with(b"WRC11BLB")
                || media[inner + 8] != worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3 as u8
                || !media[outer..].starts_with(b"WRC5FRM\0")
                || media[outer + 8] != 4
            {
                continue;
            }
            let outer_payload =
                u32::from_le_bytes(media[outer + 24..outer + 28].try_into().unwrap()) as usize;
            let inner_payload =
                u32::from_le_bytes(media[inner + 12..inner + 16].try_into().unwrap()) as usize;
            let outer_len = C5_HEADER + outer_payload;
            let inner_len = C11_HEADER + inner_payload;
            if outer + outer_len <= media.len() && inner + inner_len <= outer + outer_len {
                candidates.push((path.clone(), outer, outer_len, inner, inner_len));
            }
        }
    }
    assert_eq!(
        candidates.len(),
        1,
        "one V3 descriptor selected after first C8"
    );
    let (path, outer, outer_len, inner, inner_len) = candidates.pop().unwrap();
    let mut media = std::fs::read(&path).unwrap();
    let frame = &mut media[inner..inner + inner_len];
    frame[inner_len - 16 - 8 * 32] ^= 0x80; // source-root custody digest
    let digest = sha2::Sha256::digest([&frame[..16], &frame[C11_HEADER..]].concat());
    frame[16..C11_HEADER].copy_from_slice(&digest);
    worth_store_physical_format::BlobReclaimDescriptorV3::decode(frame).expect("mutant remains valid V3 syntax");
    let outer_frame = &mut media[outer..outer + outer_len];
    let checksum = crc32c(&outer_frame[..44], &outer_frame[C5_HEADER..]);
    outer_frame[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
}

pub(crate) fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}