use super::{ArtifactGranule, ArtifactInventory, Op};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn edit(
    bytes: &mut Vec<u8>,
    baseline: &Path,
    inventory: &ArtifactInventory,
    index: usize,
    operator: Op,
) {
    let target = &inventory.granules[index];
    let start = target.offset();
    let mut length = target.length();
    if operator == Op::ScopeSubstitution && target.family == "checkpoint_binding" {
        // A real independently valid foreign kind, not an oversized kind-byte lie.
        let donor = inventory
            .granules
            .iter()
            .find(|g| g.path == target.path && g.family == "checkpoint_binding_compaction")
            .unwrap();
        let source = std::fs::read(baseline.join(&donor.path)).unwrap();
        assert_eq!(donor.length(), 36);
        bytes.splice(
            start..start + length,
            source[donor.offset()..donor.offset() + 36].iter().copied(),
        );
        length = 36;
    } else {
        let frame = &mut bytes[start..start + length];
        match operator {
            Op::CoveredByte => frame[16] ^= 1,
            Op::Checksum => frame[length - 4] ^= 1,
            Op::Length => {
                let payload = if target.family == "checkpoint_binding" {
                    4097
                } else {
                    (length - 20) as u32 + 1
                };
                frame[12..16].copy_from_slice(&payload.to_le_bytes());
            }
            Op::EnvelopeVersion => frame[8] = 2,
            Op::ScopeSubstitution if target.family == "checkpoint_footer" => {
                let sequence = u64::from_le_bytes(frame[32..40].try_into().unwrap());
                frame[32..40].copy_from_slice(&(sequence + 1).to_le_bytes());
            }
            Op::ScopeSubstitution => {
                // Every nonempty <=4096 byte payload is an independently valid
                // opaque binding record; the stream requires this original kind.
                frame[9] = 4;
            }
            Op::SelectiveAggregate => {
                let offset = match target.family {
                    "checkpoint_dirty_basis" => 56, // lawful generation, not coordinate framing
                    "checkpoint_binding" => length - 5, // opaque C9 body
                    "checkpoint_footer" => 48,      // selected stored dirty aggregate only
                    _ => unreachable!(),
                };
                frame[offset] ^= 1;
            }
            _ => unreachable!("declared checkpoint operator"),
        }
        if !matches!(operator, Op::CoveredByte | Op::Checksum) {
            seal(frame);
        }
    }
    // B refreshes nothing. SelectiveAggregate intentionally leaves the stored
    // selective summary false. All other edits repair only enclosing layers.
    if !matches!(operator, Op::CoveredByte | Op::SelectiveAggregate) {
        refresh_enclosing(bytes, inventory, target, length);
    }
}

fn refresh_enclosing(
    bytes: &mut [u8],
    inventory: &ArtifactInventory,
    target: &ArtifactGranule,
    new_length: usize,
) {
    let delta = new_length as isize - target.length() as isize;
    let shifted = |g: &ArtifactGranule| {
        if g.offset() > target.offset() {
            (g.offset() as isize + delta) as usize
        } else {
            g.offset()
        }
    };
    let mut dirty = Sha256::new();
    let mut bindings = Sha256::new();
    let mut binding_bytes = 0u64;
    let mut footer = None;
    let mut compaction = None;
    for granule in inventory.granules.iter().filter(|g| g.path == target.path) {
        let start = shifted(granule);
        let length = if granule.offset() == target.offset() {
            new_length
        } else {
            granule.length()
        };
        match granule.family {
            "checkpoint_dirty_basis" => dirty.update(&bytes[start..start + length]),
            "checkpoint_binding" => {
                bindings.update(&bytes[start..start + length]);
                binding_bytes += length as u64;
            }
            "checkpoint_binding_compaction" => compaction = Some(start as u64),
            "checkpoint_footer" => footer = Some((start, length)),
            _ => {}
        }
    }
    let (start, length) = footer.unwrap();
    if target.family == "checkpoint_footer" {
        return;
    }
    let footer = &mut bytes[start..start + length];
    footer[48..80].copy_from_slice(&dirty.finalize());
    footer[80..88].copy_from_slice(&compaction.unwrap().to_le_bytes());
    footer[112..120].copy_from_slice(&binding_bytes.to_le_bytes());
    footer[120..152].copy_from_slice(&bindings.finalize());
    seal(footer);
}

fn seal(frame: &mut [u8]) {
    let end = frame.len() - 4;
    let crc = checksum(&frame[..end]);
    frame[end..].copy_from_slice(&crc.to_le_bytes());
}
fn checksum(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f63b78 & (crc & 1).wrapping_neg());
        }
    }
    !crc
}
