use std::path::Path;

use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange, PersistedRecordIdentity};

#[derive(Clone, Copy)]
pub(super) struct RoutedExtent {
    pub(super) record: PersistedRecordIdentity,
    pub(super) range: ExtentArenaRange,
    pub(super) generation: u64,
    pub(super) extent: u64,
    pub(super) payload_bytes: u64,
}

/// Literal C11 root-tree leaf walker for the crash oracle. It does not use
/// recovery's placement decoder or infer a file from an extent generation.
pub(super) fn selected_routes(root: &Path) -> Vec<RoutedExtent> {
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let generation = number(&catalog, 64);
    let manifest = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    assert_eq!(manifest.len(), 384);
    assert_eq!(manifest[88], 1);
    let mut pending = vec![manifest[96..168].to_vec()];
    let mut selected = Vec::new();
    while let Some(reference) = pending.pop() {
        let generation = number(&reference, 0);
        let block = number(&reference, 8);
        let bytes = std::fs::read(root.join(format!(
            "families/records/roots/root-{generation:016x}-block-{block:016x}.manifest"
        )))
        .unwrap();
        assert_eq!(
            crc32c(&bytes),
            u32::from_le_bytes(reference[20..24].try_into().unwrap())
        );
        assert_eq!(&bytes[10..12], &2_u16.to_le_bytes());
        let count = usize::from(u16::from_le_bytes(bytes[66..68].try_into().unwrap()));
        match bytes[68] {
            1 => {
                assert_eq!(bytes.len(), 88 + count * 88);
                for entry in bytes[88..].chunks_exact(88) {
                    if entry[24] != 2 {
                        continue;
                    }
                    let record = PersistedRecordIdentity::new(
                        entry[..16].try_into().unwrap(),
                        number(entry, 16),
                    )
                    .unwrap();
                    let range = ExtentArenaRange::new(
                        ExtentArenaId::new(number(entry, 32)).unwrap(),
                        number(entry, 56),
                        number(entry, 64),
                    )
                    .unwrap();
                    selected.push(RoutedExtent {
                        record,
                        range,
                        extent: number(entry, 40),
                        generation: number(entry, 48),
                        payload_bytes: number(entry, 72),
                    });
                }
            }
            2 => {
                assert_eq!(bytes.len(), 88 + count * 72);
                pending.extend(bytes[88..].chunks_exact(72).map(<[u8]>::to_vec));
            }
            _ => panic!("unknown C11 root-tree node kind"),
        }
    }
    selected
}

fn number(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
