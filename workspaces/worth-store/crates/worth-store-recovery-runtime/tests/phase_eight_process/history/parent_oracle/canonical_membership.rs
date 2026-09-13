use std::collections::{BTreeMap, BTreeSet};

use super::canonical_membership_frame::{decode_frame, find_file};
use super::canonical_membership_placement::{
    parse_placement, read_placement, Placement, RecordIdentity,
};
use super::{read_u16, read_u64};

const ROOT_SELECTOR_KIND: u8 = 11;
const ROOT_MANIFEST_KIND: u8 = 2;
const ROOT_ROUTING_KIND: u8 = 8;
const LEAF_ENTRY_BYTES: usize = 88;
const BRANCH_ENTRY_BYTES: usize = 72;

#[derive(Debug, Clone, Copy)]
struct RootReference {
    generation: u64,
    block: u64,
    level: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedCanonicalRecord {
    pub(crate) allocation_epoch: [u8; 16],
    pub(crate) ordinal: u64,
    pub(crate) payload: Vec<u8>,
    pub(crate) redo_digest: [u8; 32],
}

pub(crate) fn current_root_payloads(
    files: &[(String, Vec<u8>)],
) -> Result<BTreeSet<Vec<u8>>, String> {
    Ok(current_root_records(files)?.into_values().collect())
}

pub(crate) fn current_root_records(
    files: &[(String, Vec<u8>)],
) -> Result<BTreeMap<RecordIdentity, Vec<u8>>, String> {
    let selector = find_file(files, "families/records/root-current.selector")
        .ok_or_else(|| "parent oracle cannot find the current selector".to_owned())?;
    let selector = decode_frame(selector)
        .ok_or_else(|| "parent oracle cannot decode the current selector".to_owned())?;
    if selector.kind != ROOT_SELECTOR_KIND
        || selector.payload.len() != 59
        || selector.payload[16] != 1
    {
        return Err("parent oracle current selector is not a fixed-role selector".to_owned());
    }
    let generation = read_u64(selector.payload, 17)
        .ok_or_else(|| "parent oracle current selector omitted its generation".to_owned())?;
    let manifest_path = format!("families/records/roots/root-{generation:016x}.manifest");
    let manifest = find_file(files, &manifest_path).ok_or_else(|| {
        format!("parent oracle cannot find selected root manifest {manifest_path}")
    })?;
    let manifest = decode_frame(manifest)
        .ok_or_else(|| "parent oracle cannot decode the selected root manifest".to_owned())?;
    if manifest.kind != ROOT_MANIFEST_KIND
        || manifest.identity != generation
        || manifest.payload.len() != 320
        || manifest.payload[40] != 1
    {
        return Err("parent oracle selected root manifest has no valid routing root".to_owned());
    }
    let root = RootReference {
        generation: read_u64(manifest.payload, 48)
            .ok_or_else(|| "parent oracle routing reference omitted generation".to_owned())?,
        block: read_u64(manifest.payload, 56)
            .ok_or_else(|| "parent oracle routing reference omitted block".to_owned())?,
        level: read_u16(manifest.payload, 64)
            .ok_or_else(|| "parent oracle routing reference omitted level".to_owned())?,
    };
    let mut visited = BTreeSet::new();
    let mut placements = Vec::new();
    visit_root_block(files, root, &mut visited, &mut placements)?;
    let mut canonical_records = BTreeMap::new();
    for placement in placements {
        let (record, payload) = read_placement(files, placement)?;
        if canonical_records.insert(record, payload).is_some() {
            return Err(
                "parent oracle current root contains a duplicate record identity".to_owned(),
            );
        }
    }
    Ok(canonical_records)
}

pub fn require_current_root_membership(
    files: &[(String, Vec<u8>)],
    expected: &BTreeMap<[u8; 32], ExpectedCanonicalRecord>,
) -> Result<(), String> {
    let canonical_records = current_root_records(files)?;
    require_expected_records(&canonical_records, expected)?;
    super::in_flight::require_bound_records(files, expected)?;
    if canonical_records.len() != expected.len() {
        let expected_identities = expected
            .values()
            .map(|record| RecordIdentity {
                allocation_epoch: record.allocation_epoch,
                ordinal: record.ordinal,
            })
            .collect::<BTreeSet<_>>();
        let actual_identities = canonical_records.keys().copied().collect::<BTreeSet<_>>();
        let extra = actual_identities
            .difference(&expected_identities)
            .map(|identity| format!("{:02x?}/{}", identity.allocation_epoch, identity.ordinal))
            .collect::<Vec<_>>();
        let missing = expected_identities
            .difference(&actual_identities)
            .map(|identity| format!("{:02x?}/{}", identity.allocation_epoch, identity.ordinal))
            .collect::<Vec<_>>();
        return Err(format!(
            "parent history canonical current-root membership count mismatch: actual {}, expected {}; extra={extra:?}; missing={missing:?}",
            canonical_records.len(),
            expected.len()
        ));
    }
    Ok(())
}

pub(crate) fn require_current_root_membership_with_unresolved_payload(
    files: &[(String, Vec<u8>)],
    expected: &BTreeMap<[u8; 32], ExpectedCanonicalRecord>,
    unresolved_idempotency: &[u8],
    unresolved_payload: &[u8],
) -> Result<bool, String> {
    let canonical_records = current_root_records(files)?;
    require_expected_records(&canonical_records, expected)?;
    super::in_flight::require_bound_records(files, expected)?;
    let expected_identities = expected
        .values()
        .map(|record| RecordIdentity {
            allocation_epoch: record.allocation_epoch,
            ordinal: record.ordinal,
        })
        .collect::<BTreeSet<_>>();
    let unresolved = canonical_records
        .iter()
        .filter(|(identity, _)| !expected_identities.contains(identity))
        .collect::<Vec<_>>();
    if unresolved.len() > 1 {
        return Err(format!(
            "parent history unresolved recovery membership is ambiguous: extra_count={}",
            unresolved.len(),
        ));
    }
    let Some((record, observed_payload)) = unresolved.first() else {
        return Ok(false);
    };
    if observed_payload.as_slice() != unresolved_payload {
        return Err(
            "parent history unresolved recovery record payload disagrees with the dirty operation"
                .to_owned(),
        );
    }
    super::in_flight::require_bound_record(
        files,
        unresolved_idempotency,
        **record,
        unresolved_payload,
        None,
    )?;
    Ok(true)
}

fn require_expected_records(
    canonical_records: &BTreeMap<RecordIdentity, Vec<u8>>,
    expected: &BTreeMap<[u8; 32], ExpectedCanonicalRecord>,
) -> Result<(), String> {
    if canonical_records.len() < expected.len() {
        return Err(format!(
            "parent history canonical current-root membership count below expected: actual {}, expected {}",
            canonical_records.len(),
            expected.len()
        ));
    }
    for (index, expected) in expected.values().enumerate() {
        let identity = RecordIdentity {
            allocation_epoch: expected.allocation_epoch,
            ordinal: expected.ordinal,
        };
        if canonical_records.get(&identity) != Some(&expected.payload) {
            return Err(format!(
                "parent history canonical current-root membership identity/payload mismatch at {index}"
            ));
        }
    }
    Ok(())
}

fn visit_root_block(
    files: &[(String, Vec<u8>)],
    reference: RootReference,
    visited: &mut BTreeSet<(u64, u64)>,
    placements: &mut Vec<Placement>,
) -> Result<(), String> {
    if reference.generation == 0
        || reference.block == 0
        || !visited.insert((reference.generation, reference.block))
    {
        return Err("parent oracle routing root repeated or had a zero identity".to_owned());
    }
    let path = format!(
        "families/records/roots/root-{generation:016x}-block-{block:016x}.manifest",
        generation = reference.generation,
        block = reference.block
    );
    let bytes = find_file(files, &path)
        .ok_or_else(|| format!("parent oracle cannot find selected routing block {path}"))?;
    let frame = decode_frame(bytes)
        .ok_or_else(|| format!("parent oracle cannot decode selected routing block {path}"))?;
    if frame.kind != ROOT_ROUTING_KIND || frame.identity != reference.block {
        return Err(format!(
            "parent oracle routing block identity mismatch {path}"
        ));
    }
    let payload = frame.payload;
    let level = read_u16(payload, 16).ok_or_else(|| "routing prefix is truncated".to_owned())?;
    let count =
        usize::from(read_u16(payload, 18).ok_or_else(|| "routing count is truncated".to_owned())?);
    let kind = *payload
        .get(20)
        .ok_or_else(|| "routing kind is truncated".to_owned())?;
    if payload.len() < 40
        || read_u64(payload, 8) != Some(reference.block)
        || level != reference.level
        || count == 0
    {
        return Err(format!(
            "parent oracle routing block prefix is invalid {path}"
        ));
    }
    match (kind, level) {
        (1, 0) => {
            let end = 40usize
                .checked_add(
                    count
                        .checked_mul(LEAF_ENTRY_BYTES)
                        .ok_or("leaf count overflow")?,
                )
                .ok_or("leaf routing length overflow")?;
            if payload.len() != end {
                return Err(format!(
                    "parent oracle leaf routing length is invalid {path}"
                ));
            }
            for entry in payload[40..].chunks_exact(LEAF_ENTRY_BYTES) {
                placements.push(parse_placement(entry)?);
            }
        }
        (2, level) if level != 0 => {
            let end = 40usize
                .checked_add(
                    count
                        .checked_mul(BRANCH_ENTRY_BYTES)
                        .ok_or("branch count overflow")?,
                )
                .ok_or("branch routing length overflow")?;
            if payload.len() != end {
                return Err(format!(
                    "parent oracle branch routing length is invalid {path}"
                ));
            }
            for child in payload[40..].chunks_exact(BRANCH_ENTRY_BYTES) {
                if child[18..20] != [0; 2] {
                    return Err(
                        "parent oracle branch reference reserved bytes are nonzero".to_owned()
                    );
                }
                visit_root_block(
                    files,
                    RootReference {
                        generation: read_u64(child, 0).ok_or("branch generation is truncated")?,
                        block: read_u64(child, 8).ok_or("branch block is truncated")?,
                        level: read_u16(child, 16).ok_or("branch level is truncated")?,
                    },
                    visited,
                    placements,
                )?;
            }
        }
        _ => {
            return Err(format!(
                "parent oracle routing kind/level is invalid {path}"
            ));
        }
    }
    Ok(())
}
