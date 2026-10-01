use super::super::record_walk::damage;
use super::super::{
    families::{durable_frame::read_u16, physical_fields::record_key},
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnsupportedPhysicalVersion,
    OfflineUnsupportedVersionAxis,
};
use super::resolver::RecordKey;
use worth_foundational::PhysicalByteRange;

pub(super) struct DirectoryEntry {
    pub(super) family_code: u16,
    pub(super) root: RecordKey,
}

pub(super) fn inspect_directory(
    bytes: &[u8],
    selected_root: &[u8],
) -> Result<Vec<DirectoryEntry>, Outcome> {
    if selected_root.len() < 528 {
        return Err(damage(Cause::Framing, None, Blast::Artifact));
    }
    if bytes.len() < 76 || &bytes[..8] != b"WRC11IDX" {
        return Err(damage(
            Cause::Framing,
            Some((0, bytes.len().min(8).max(1) as u64)),
            Blast::Artifact,
        ));
    }
    if !matches!(bytes[8], 1 | 2) {
        return Err(Outcome::Unsupported(
            OfflineUnsupportedPhysicalVersion::new(
                OfflineUnsupportedVersionAxis::PhysicalRecordFormat,
                u64::from(bytes[8]),
                "1|2",
                PhysicalByteRange::new(8, 1).unwrap(),
            ),
        ));
    }
    let prefix_bytes = if bytes[8] == 2 { 101 } else { 76 };
    let count = usize::from(bytes[9]);
    if count > 64 || bytes.len() != prefix_bytes + count * 26 || bytes[11] != 0 || bytes[10] > 1 {
        return Err(malformed());
    }
    let root_marker = &selected_root[432..496];
    let marker = &bytes[12..76];
    let marker_matches = match bytes[10] {
        0 => marker == [0; 64] && root_marker == [0; 64],
        1 => {
            record_key(marker).is_some()
                && u64::from_le_bytes(marker[24..32].try_into().unwrap()) != 0
                && marker == root_marker
        }
        _ => false,
    };
    if !marker_matches {
        return Err(pointer());
    }
    let selected_quarantine = (selected_root[496] == 1).then_some(&selected_root[504..528]);
    let indexed_quarantine = if prefix_bytes == 101 {
        match bytes[76] {
            0 if bytes[77..101] == [0; 24] => None,
            1 if record_key(&bytes[77..101]).is_some() => Some(&bytes[77..101]),
            _ => return Err(malformed()),
        }
    } else {
        None
    };
    if indexed_quarantine != selected_quarantine {
        return Err(pointer());
    }
    let mut entries = Vec::with_capacity(count);
    let mut previous = 0;
    for frame in bytes[prefix_bytes..].chunks_exact(26) {
        let code = read_u16(frame, 0);
        let record = record_key(&frame[2..26]).ok_or_else(pointer)?;
        if !matches!(code, 1 | 2) || code <= previous {
            return Err(malformed());
        }
        entries.push(DirectoryEntry {
            family_code: code,
            root: record,
        });
        previous = code;
    }
    let has_catalog = entries.iter().any(|entry| entry.family_code == 1);
    if has_catalog && (selected_root[401] != 1 || root_marker != &selected_root[336..400]) {
        return Err(pointer());
    }
    Ok(entries)
}

fn malformed() -> Outcome {
    damage(Cause::MalformedPayload, None, Blast::Artifact)
}
fn pointer() -> Outcome {
    damage(Cause::Pointer, None, Blast::ReachableRootSubtree)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(ordinal: u64) -> [u8; 24] {
        let mut record = [1; 24];
        record[16..24].copy_from_slice(&ordinal.to_le_bytes());
        record
    }

    #[test]
    fn literal_directory_requires_exact_selected_marker_and_ordered_roots() {
        let mut selected = [0; 528];
        selected[401] = 1;
        selected[336..360].copy_from_slice(&record(7));
        selected[360..368].copy_from_slice(&3_u64.to_le_bytes());
        selected[368..400].fill(9);
        let marker: [u8; 64] = selected[336..400].try_into().unwrap();
        selected[432..496].copy_from_slice(&marker);
        let mut bytes = vec![0; 76 + 2 * 26];
        bytes[..8].copy_from_slice(b"WRC11IDX");
        bytes[8] = 1;
        bytes[9] = 2;
        bytes[10] = 1;
        bytes[12..76].copy_from_slice(&marker);
        bytes[76..78].copy_from_slice(&1_u16.to_le_bytes());
        bytes[78..102].copy_from_slice(&record(8));
        bytes[102..104].copy_from_slice(&2_u16.to_le_bytes());
        bytes[104..128].copy_from_slice(&record(9));
        assert_eq!(inspect_directory(&bytes, &selected).unwrap().len(), 2);
        selected[432] ^= 1;
        assert!(matches!(
            inspect_directory(&bytes, &selected),
            Err(Outcome::Damaged(_))
        ));
    }

    #[test]
    fn directory_rejects_duplicate_families_and_stale_catalog_marker() {
        let mut selected = [0; 528];
        let mut bytes = vec![0; 76 + 2 * 26];
        bytes[..8].copy_from_slice(b"WRC11IDX");
        bytes[8] = 1;
        bytes[9] = 2;
        bytes[76..78].copy_from_slice(&2_u16.to_le_bytes());
        bytes[78..102].copy_from_slice(&record(8));
        bytes[102..104].copy_from_slice(&2_u16.to_le_bytes());
        bytes[104..128].copy_from_slice(&record(9));
        assert!(inspect_directory(&bytes, &selected).is_err());
        bytes[76..78].copy_from_slice(&1_u16.to_le_bytes());
        selected[401] = 0;
        assert!(inspect_directory(&bytes, &selected).is_err());
    }

    #[test]
    fn v2_directory_quarantine_marker_must_match_selected_root() {
        let mut selected = [0; 528];
        selected[496] = 1;
        selected[504..528].copy_from_slice(&record(7));
        let mut bytes = vec![0; 101 + 26];
        bytes[..8].copy_from_slice(b"WRC11IDX");
        bytes[8] = 2;
        bytes[9] = 1;
        bytes[76] = 1;
        bytes[77..101].copy_from_slice(&record(7));
        bytes[101..103].copy_from_slice(&2_u16.to_le_bytes());
        bytes[103..127].copy_from_slice(&record(8));
        assert_eq!(inspect_directory(&bytes, &selected).unwrap().len(), 1);
        bytes[77..101].copy_from_slice(&record(9));
        assert!(matches!(
            inspect_directory(&bytes, &selected),
            Err(Outcome::Damaged(_))
        ));
        bytes[77..101].copy_from_slice(&record(7));
        selected[496] = 0;
        selected[504..528].fill(0);
        assert!(matches!(
            inspect_directory(&bytes, &selected),
            Err(Outcome::Damaged(_))
        ));
    }
}
