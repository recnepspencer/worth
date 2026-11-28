//! Exact generation-matched Native checkpoint extraction from a Directory Home.
use super::directory_lease;
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;

/// Locate the exact `WSANREC2` anchored record for `home.checkpoint` at
/// `expected_generation`, decode its `WAPHCKP1` Relational section, and
/// extract the Native `WORTH-REL-CHK25` frame without falling back to older
/// checkpoint generations.
pub(super) fn read_native_checkpoint_frame_for_generation(
    root: &Path,
    expected_generation: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Option<(u64, u64, Vec<u8>)> {
    const RECORD_MAGIC: &[u8; 8] = b"WSANREC2";
    const ANCHOR_NAME_BYTES: usize = 32;
    const HEADER_BYTES: usize = 8 + ANCHOR_NAME_BYTES + 8 + 32 + 8 + 8;
    const MEMBER_LOCATOR_BYTES: usize = 64;
    const MANIFEST_MAGIC: &[u8; 8] = b"WAPHCKP1";
    const MANIFEST_HEADER_BYTES: usize = 44;

    let mut expected_anchor = [0u8; ANCHOR_NAME_BYTES];
    let anchor_name = b"home.checkpoint";
    expected_anchor[..anchor_name.len()].copy_from_slice(anchor_name);

    let mut files = Vec::new();
    collect_files_recursive(root, &mut files);
    files.sort();
    for path in files {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let mut search = 0;
        while let Some(pos) = data[search..]
            .windows(RECORD_MAGIC.len())
            .position(|window| window == RECORD_MAGIC)
        {
            let at = search + pos;
            search = at + RECORD_MAGIC.len();
            let Some(header) = data.get(at..at + HEADER_BYTES) else {
                continue;
            };
            if header[8..40] != expected_anchor {
                continue;
            }
            let generation = u64::from_le_bytes(header[40..48].try_into().expect("fixed"));
            if generation != expected_generation {
                continue;
            }
            let payload_bytes = usize::try_from(u64::from_le_bytes(
                header[80..88].try_into().expect("fixed"),
            ))
            .ok()?;
            let member_count = usize::try_from(u64::from_le_bytes(
                header[88..96].try_into().expect("fixed"),
            ))
            .ok()?;
            let table_bytes = member_count.checked_mul(MEMBER_LOCATOR_BYTES)?;
            let payload_start = at.checked_add(HEADER_BYTES)?.checked_add(table_bytes)?;
            let payload_end = payload_start.checked_add(payload_bytes)?;
            let Some(payload) = data.get(payload_start..payload_end) else {
                continue;
            };
            if payload.len() < MANIFEST_HEADER_BYTES || &payload[..8] != MANIFEST_MAGIC {
                continue;
            }
            let version = u32::from_le_bytes(payload[8..12].try_into().expect("fixed"));
            let section_count = u16::from_le_bytes(payload[12..14].try_into().expect("fixed"));
            if version != 3 || section_count != 3 {
                continue;
            }
            let relational_tag = u16::from_le_bytes(payload[14..16].try_into().expect("fixed"));
            let relational_len = usize::try_from(u64::from_le_bytes(
                payload[16..24].try_into().expect("fixed"),
            ))
            .ok()?;
            if relational_tag != worth_foundational::facade::DurableOwnerFamily::Relational.tag()
                || relational_len == 0
            {
                continue;
            }
            let Some(relational_section) = payload
                .get(MANIFEST_HEADER_BYTES..MANIFEST_HEADER_BYTES.checked_add(relational_len)?)
            else {
                continue;
            };
            if let Some(decoded) =
                extract_native_from_admission_section(relational_section, request)
            {
                return Some(decoded);
            }
        }
    }
    None
}

fn collect_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn extract_native_from_admission_section(
    slice: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Option<(u64, u64, Vec<u8>)> {
    const ADMISSION_HEADER: usize = 292;
    if slice.len() < ADMISSION_HEADER || &slice[..8] != b"WSRCAI01" {
        return None;
    }
    let version = u32::from_le_bytes(slice[8..12].try_into().ok()?);
    if version != 1 {
        return None;
    }
    let covered = u64::from_le_bytes(slice[12..20].try_into().ok()?);
    let transitions = u64::from_le_bytes(slice[20..28].try_into().ok()?);
    let source_len = usize::try_from(u64::from_le_bytes(slice[172..180].try_into().ok()?)).ok()?;
    let image_len = usize::try_from(u64::from_le_bytes(slice[180..188].try_into().ok()?)).ok()?;
    if source_len == 0 || image_len == 0 {
        return None;
    }
    let source_end = ADMISSION_HEADER.checked_add(source_len)?;
    let image_end = source_end.checked_add(image_len)?;
    if slice.len() < image_end {
        return None;
    }
    let mut builder =
        worth_execution::ExecutionRequestArrayBuilder::allocate(*request, image_end).ok()?;
    builder.extend_from_slice(&slice[..image_end]).ok()?;
    let backing = builder.seal().ok()?.into_immutable_bytes();
    let source = worth_relational::facade::durability::RelationalCompleteSourceArtifact::from_untrusted_region(
        backing.clone(),
        ADMISSION_HEADER..source_end,
        request,
    )
    .ok()?;
    let image = worth_relational::facade::durability::RelationalCheckpointAdmissionArtifact::from_untrusted_region(
        backing,
        source_end..image_end,
        request,
    )
    .ok()?;
    let checkpoint =
        worth_relational::facade::durability::RelationalDurableCheckpointSource::from_checkpoint_admission_image(
            source,
            image,
            request,
        )
        .ok()?;
    Some((
        covered,
        transitions,
        checkpoint.checkpoint().bytes().to_vec(),
    ))
}

pub fn inspect_native_checkpoint_frame(
    frame: &[u8],
) -> worth_relational::facade::durability::RelationalCheckpointInspection<'_> {
    let lease = directory_lease();
    let request = ExecutionRequest::leased(&lease);
    worth_relational::facade::durability::RelationalNativeCheckpoint::inspect_bytes(&request, frame)
        .expect("inspect published Native checkpoint frame")
}

pub fn assert_native_checkpoint_model_unchanged(before: &[u8], after: &[u8]) {
    let old = inspect_native_checkpoint_frame(before);
    let now = inspect_native_checkpoint_frame(after);
    assert_eq!(
        now.model, old.model,
        "all Native model and publication value bytes are unchanged"
    );
    assert_eq!(now.live_records, old.live_records);
    assert_eq!(
        now.current_publication_records,
        old.current_publication_records
    );
}

pub fn assert_native_checkpoint_fully_unchanged(before: &[u8], after: &[u8]) {
    let old = inspect_native_checkpoint_frame(before);
    let now = inspect_native_checkpoint_frame(after);
    assert_eq!(
        now.model, old.model,
        "all Native model/publication/symbol value bytes remain equal"
    );
    assert_eq!(
        now.live_records, old.live_records,
        "no receipt or other live record was created"
    );
    assert_eq!(
        now.current_publication_records,
        old.current_publication_records
    );
    assert_eq!(now.frontiers, old.frontiers, "no Native record allocation");
    assert_eq!(now.generations, old.generations, "no Native identity reuse");
    assert_eq!(
        now.record_allocation_bytes, old.record_allocation_bytes,
        "no Native record allocation or identity page-roster advance"
    );
    assert_eq!(
        now.symbol_identity_frontier, old.symbol_identity_frontier,
        "no receipt symbols or other symbol identities allocated"
    );
}
