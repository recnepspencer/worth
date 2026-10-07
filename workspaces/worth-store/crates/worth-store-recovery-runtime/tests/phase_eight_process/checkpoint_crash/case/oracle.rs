use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::super::history;

#[path = "oracle/selected_wal.rs"]
mod selected_wal;

pub(super) fn independently_classify_in_flight(
    root: &Path,
    expected: &history::ExpectedWriterHistory,
) -> history::InFlightMutationFate {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    let idempotency = expected
        .dirty_idempotency()
        .expect("checkpoint oracle requires persisted dirty-operation binding");
    fate_from_files(&files, &idempotency, expected.in_flight_payload())
}

fn fate_from_files(
    files: &[(String, Vec<u8>)],
    idempotency: &[u8; 32],
    payload: &[u8],
) -> history::InFlightMutationFate {
    let selected_root = files
        .iter()
        .any(|(path, _)| path == "families/records/root-current.selector")
        && history::current_root_payloads(files)
            .expect("selected root must have a valid, fully routed record set")
            .contains(payload);
    let bound_wal = selected_wal::tail(files).is_some_and(|tail| {
        tail.into_iter()
            .any(|(path, bytes)| wal_contains_bound_payload(path, bytes, idempotency, payload))
    });
    if selected_root || bound_wal {
        history::InFlightMutationFate::DurableEffect
    } else {
        // A staged arena may contain a byte-for-byte payload before any
        // selected route or durable WAL binding. Its raw bytes prove no fate.
        history::InFlightMutationFate::Indeterminate
    }
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    let entries =
        std::fs::read_dir(directory).expect("read independent checkpoint oracle directory");
    for entry in entries {
        let entry = entry.expect("read independent checkpoint oracle entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("independent checkpoint oracle path is beneath root")
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(&path).expect("read independent checkpoint oracle artifact");
            files.push((relative, bytes));
        }
    }
}

fn wal_contains_bound_payload(
    path: &str,
    bytes: &[u8],
    idempotency: &[u8; 32],
    payload: &[u8],
) -> bool {
    const HEADER_BYTES: usize = 116;
    const FOOTER_BYTES: usize = 32;
    let Some((path_segment, path_generation)) = wal_path_identity(path) else {
        return false;
    };
    let mut offset = 0;
    let mut previous_end = None;
    while offset < bytes.len() {
        let Some(header) = bytes.get(offset..offset + HEADER_BYTES) else {
            return false;
        };
        let (Some(segment), Some(generation), Some(start), Some(end)) = (
            read_u64(header, 12),
            read_u64(header, 20),
            read_u64(header, 28),
            read_u64(header, 36),
        ) else {
            return false;
        };
        if &header[..8] != b"WORTHWAL"
            || read_u16(header, 8) != Some(1)
            || read_u16(header, 10) != Some(HEADER_BYTES as u16)
            || segment != path_segment
            || generation != path_generation
            || segment == 0
            || generation == 0
            || start >= end
            || previous_end.is_some_and(|prior| prior != start)
        {
            return false;
        }
        let Some(payload_bytes) =
            read_u64(header, 44).and_then(|value| usize::try_from(value).ok())
        else {
            return false;
        };
        if payload_bytes == 0 {
            return false;
        }
        let Some(total) = HEADER_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| value.checked_add(FOOTER_BYTES))
        else {
            return false;
        };
        let Some(frame) = bytes.get(offset..offset + total) else {
            return false;
        };
        let frame_payload = &frame[HEADER_BYTES..HEADER_BYTES + payload_bytes];
        if Sha256::digest(frame_payload)[..] != header[84..116]
            || Sha256::digest(&frame[..HEADER_BYTES + payload_bytes])[..]
                != frame[HEADER_BYTES + payload_bytes..]
        {
            return false;
        }
        let Some((binding, remaining)) = take_field(frame_payload) else {
            return false;
        };
        let Some((redo, remaining)) = take_field(remaining) else {
            return false;
        };
        if remaining.is_empty()
            && binding_matches_redo(binding, idempotency, redo, (start, end), &header[52..84])
            && canonical_redo_matches_payload(redo, payload)
        {
            return true;
        }
        offset += total;
        previous_end = Some(end);
    }
    false
}

fn wal_path_identity(path: &str) -> Option<(u64, u64)> {
    let name = path
        .strip_prefix("families/wal/segment-")?
        .strip_suffix(".wal")?;
    let (segment, generation) = name.split_once("-generation-")?;
    let segment = segment.parse::<u64>().ok()?;
    let generation = generation.parse::<u64>().ok()?;
    (segment > 0
        && generation > 0
        && path == format!("families/wal/segment-{segment}-generation-{generation}.wal"))
    .then_some((segment, generation))
}

fn binding_matches_redo(
    binding: &[u8],
    idempotency: &[u8; 32],
    redo: &[u8],
    frame_lsn: (u64, u64),
    declared_identity_digest: &[u8],
) -> bool {
    const ATTEMPT_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";
    const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";
    let Some((encoded_domain, remaining)) = take_field(binding) else {
        return false;
    };
    let Some((encoded_identity, remaining)) = take_field(remaining) else {
        return false;
    };
    if encoded_domain != ATTEMPT_DOMAIN || encoded_identity != idempotency {
        return false;
    }
    let Some((store, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((policy, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((issuance, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((expiry, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((material, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((fingerprint, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((mutation_store, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((runtime, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((lifecycle, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((operation, mut remaining)) = take_u64(remaining) else {
        return false;
    };
    if store.len() != 16
        || policy.len() != 32
        || material.len() != 32
        || fingerprint.len() != 32
        || store.iter().all(|byte| *byte == 0)
        || policy.iter().all(|byte| *byte == 0)
        || material.iter().all(|byte| *byte == 0)
        || fingerprint.iter().all(|byte| *byte == 0)
        || mutation_store != store
        || issuance >= expiry
        || runtime == 0
        || lifecycle == 0
        || operation == 0
    {
        return false;
    }
    let mut key = Sha256::new();
    key.update((KEY_DOMAIN.len() as u64).to_le_bytes());
    key.update(KEY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material);
    if key.finalize()[..] != idempotency[..] {
        return false;
    }
    let Some((group_identity, rest)) = take_field(remaining) else {
        return false;
    };
    if group_identity.len() != 32 || group_identity.iter().all(|byte| *byte == 0) {
        return false;
    }
    remaining = rest;
    let Some((ordinal, rest)) = take_u32(remaining) else {
        return false;
    };
    let Some((member_count, rest)) = take_u32(rest) else {
        return false;
    };
    if ordinal == 0 || member_count == 0 || ordinal > member_count {
        return false;
    }
    remaining = rest;
    let mut member_identity = None;
    for ordinal in 0..2 {
        let Some((field, rest)) = take_field(remaining) else {
            return false;
        };
        if field.len() != 32 || field.iter().all(|byte| *byte == 0) {
            return false;
        }
        if ordinal == 1 {
            member_identity = Some(field);
        }
        remaining = rest;
    }
    let Some((start, rest)) = take_u64(remaining) else {
        return false;
    };
    let Some((end, rest)) = take_u64(rest) else {
        return false;
    };
    let Some((redo_digest, trailing)) = take_field(rest) else {
        return false;
    };
    let declared_identity = format!(
        "group-{}-member-{}-{}",
        hex(group_identity),
        hex(member_identity.expect("second fixed identity field exists")),
        hex(fingerprint)
    );
    start < end
        && (start, end) == frame_lsn
        && redo_digest.len() == 32
        && trailing.is_empty()
        && Sha256::digest(declared_identity.as_bytes())[..] == declared_identity_digest[..]
        && Sha256::digest(redo)[..] == redo_digest[..]
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_redo_matches_payload(redo: &[u8], payload: &[u8]) -> bool {
    const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
    let Some((domain, remaining)) = take_field(redo) else {
        return false;
    };
    let Some((record_count, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((record_ordinal, remaining)) = take_u32(remaining) else {
        return false;
    };
    let Some((sequence, remaining)) = take_u64(remaining) else {
        return false;
    };
    let Some((target_count, mut remaining)) = take_u64(remaining) else {
        return false;
    };
    if domain != REDO_DOMAIN
        || record_count != 1
        || record_ordinal != 0
        || sequence == 0
        || target_count == 0
    {
        return false;
    }
    for _ in 0..target_count {
        let Some((target, rest)) = take_field(remaining) else {
            return false;
        };
        let Some((digest, rest)) = rest.split_at_checked(32) else {
            return false;
        };
        if target.is_empty() || digest.iter().all(|byte| *byte == 0) {
            return false;
        }
        remaining = rest;
    }
    let Some((record_payload, remaining)) = take_field(remaining) else {
        return false;
    };
    let Some((projection, trailing)) = take_field(remaining) else {
        return false;
    };
    record_payload == payload && !projection.is_empty() && trailing.is_empty()
}

fn take_field(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let length = usize::try_from(read_u64(bytes, 0)?).ok()?;
    let end = 8usize.checked_add(length)?;
    Some((bytes.get(8..end)?, bytes.get(end..)?))
}

fn take_u32(bytes: &[u8]) -> Option<(u32, &[u8])> {
    let (value, remaining) = bytes.split_at_checked(4)?;
    Some((u32::from_le_bytes(value.try_into().ok()?), remaining))
}

fn take_u64(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let (value, remaining) = bytes.split_at_checked(8)?;
    Some((u64::from_le_bytes(value.try_into().ok()?), remaining))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let value = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes(value.try_into().ok()?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)?
        .try_into()
        .ok()
        .map(u64::from_le_bytes)
}

#[cfg(test)]
#[path = "oracle_tests.rs"]
mod tests;
