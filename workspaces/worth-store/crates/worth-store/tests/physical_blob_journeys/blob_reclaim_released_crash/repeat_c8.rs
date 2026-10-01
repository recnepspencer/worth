//! The first recovery publishes a V3 drop without folding its WAL into a
//! checkpoint. A second, independent C8 process must preserve that result.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::Duration,
};

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobReclaimDescriptorV3, BlobRecordKind};

use super::{kill_at, recover_closed_store, retirement_witness, ROLE};

const C5_HEADER: usize = 48;
const C5_EXTENT_METADATA: usize = 64;
const C11_HEADER: usize = 48;

#[test]
fn second_fresh_c8_preserves_the_first_published_v3_drop() {
    let world = kill_at(ROLE, Duration::from_secs(240));
    let checkpoint = fs::read(world.root.join("families/checkpoint.current")).unwrap();
    let (source, format) = retirement_witness::selected_root(&world.root);
    let (publication, _) = retirement_witness::selected_publication_extent(&world.root, &source);
    recover_closed_store(&world.root);
    let (first, first_format) = retirement_witness::selected_root(&world.root);
    assert_eq!(first_format, format);
    let first_placements = retirement_witness::selected_placements(&world.root, &first);
    assert_eq!(first.generation(), source.generation() + 1);
    assert!(first_placements
        .iter()
        .all(|placement| placement.record() != publication));
    assert_eq!(
        fs::read(world.root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
    let output = run_recovery(&world.root);
    assert!(
        output.status.success(),
        "repeat C8 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C8_RECOVERY_RUNTIME "),
        "fresh C8 did not construct a recovered runtime: {stderr}"
    );
    assert!(
        stderr.contains("C8_RECOVERY_REDO apply=0 "),
        "repeat must not apply WAL: {stderr}"
    );
    let skip = stderr
        .lines()
        .find_map(|line| line.strip_prefix("C8_RECOVERY_REDO apply=0 skip_historical_drop="))
        .and_then(|count| count.parse::<u64>().ok())
        .expect("bounded historical skip counter");
    assert!(
        skip > 0,
        "the old dropped WAL target must be classified: {stderr}"
    );
    assert!(
        stderr.contains("fate=HistoricalReleaseConsumed"),
        "old WAL group must be terminally consumed: {stderr}"
    );
    let (second, second_format) = retirement_witness::selected_root(&world.root);
    assert_eq!(second_format, format);
    assert_eq!(
        second, first,
        "repeat C8 must not resurrect a dropped WAL target"
    );
    assert_eq!(
        retirement_witness::selected_placements(&world.root, &second),
        first_placements
    );
    assert_eq!(
        fs::read(world.root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
}

#[test]
fn selected_v3_witness_changed_after_first_c8_denies_repeat() {
    let world = kill_at(ROLE, Duration::from_secs(240));
    recover_closed_store(&world.root);
    mutate_selected_descriptor_custody(&world.root);
    let output = run_recovery(&world.root);
    assert!(
        !output.status.success(),
        "changed selected V3 custody must deny repeat C8"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C8_RECOVERY_BLOCKED kind=PageAdmission"),
        "changed selected control must fail at historical page admission: {stderr}"
    );
}

fn run_recovery(root: &Path) -> Output {
    let binary = std::env::var_os("WORTH_C8_RECOVERY_EXECUTABLE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .and_then(Path::parent)
                .unwrap()
                .join(format!(
                    "physical_store_recover{}",
                    std::env::consts::EXE_SUFFIX
                ))
        });
    Command::new(binary)
        .arg(root)
        .arg("--bounded-profile=c11-blob-crash-v1")
        .output()
        .unwrap()
}

fn mutate_selected_descriptor_custody(root: &Path) {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = fs::read(&path).unwrap();
        for inner in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer) = inner.checked_sub(C5_HEADER + C5_EXTENT_METADATA) else {
                continue;
            };
            if !media[inner..].starts_with(b"WRC11BLB")
                || media[inner + 8] != BlobRecordKind::ReclaimDescriptorV3 as u8
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
    let mut media = fs::read(&path).unwrap();
    let frame = &mut media[inner..inner + inner_len];
    frame[inner_len - 16 - 8 * 32] ^= 0x80; // source-root custody digest
    let digest = Sha256::digest([&frame[..16], &frame[C11_HEADER..]].concat());
    frame[16..C11_HEADER].copy_from_slice(&digest);
    BlobReclaimDescriptorV3::decode(frame).expect("mutant remains valid V3 syntax");
    let outer_frame = &mut media[outer..outer + outer_len];
    let checksum = crc32c(&outer_frame[..44], &outer_frame[C5_HEADER..]);
    outer_frame[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
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
