use std::path::Path;

use super::super::super::history;

pub(super) fn independently_classify_in_flight(
    root: &Path,
    expected: &history::ExpectedWriterHistory,
) -> history::InFlightMutationFate {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    let identity_present = files
        .iter()
        .any(|(_, bytes)| contains(bytes, &expected.in_flight_identity()));
    let payload_present = files.iter().any(|(path, bytes)| {
        if bytes.starts_with(b"WORTHWAL") {
            wal_contains_bound_payload(
                bytes,
                &expected.in_flight_identity(),
                expected.in_flight_payload(),
            )
        } else {
            path.starts_with("families/records/") && contains(bytes, expected.in_flight_payload())
        }
    });
    match (identity_present, payload_present) {
        (_, true) => history::InFlightMutationFate::DurableEffect,
        (false, false) | (true, false) => history::InFlightMutationFate::Indeterminate,
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

fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && bytes.windows(needle.len()).any(|window| window == needle)
}

fn wal_contains_bound_payload(bytes: &[u8], identity: &[u8], payload: &[u8]) -> bool {
    const HEADER_BYTES: usize = 116;
    const FOOTER_BYTES: usize = 32;
    const ATTEMPT_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";
    let mut offset = 0;
    while offset < bytes.len() {
        let Some(header) = bytes.get(offset..offset + HEADER_BYTES) else {
            return false;
        };
        let Some(payload_bytes) =
            read_u64(header, 44).and_then(|value| usize::try_from(value).ok())
        else {
            return false;
        };
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
        let Some((binding, remaining)) = take_field(frame_payload) else {
            return false;
        };
        let Some((redo, remaining)) = take_field(remaining) else {
            return false;
        };
        if remaining.is_empty()
            && binding_contains_identity(binding, ATTEMPT_DOMAIN, identity)
            && redo == payload
        {
            return true;
        }
        offset += total;
    }
    false
}

fn binding_contains_identity(binding: &[u8], domain: &[u8], identity: &[u8]) -> bool {
    let Some((encoded_domain, remaining)) = take_field(binding) else {
        return false;
    };
    let Some((encoded_identity, _)) = take_field(remaining) else {
        return false;
    };
    encoded_domain == domain && encoded_identity == identity
}

fn take_field(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let length = usize::try_from(read_u64(bytes, 0)?).ok()?;
    let end = 8usize.checked_add(length)?;
    Some((bytes.get(8..end)?, bytes.get(end..)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)?
        .try_into()
        .ok()
        .map(u64::from_le_bytes)
}
