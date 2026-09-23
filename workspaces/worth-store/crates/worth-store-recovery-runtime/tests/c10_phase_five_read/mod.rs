use std::path::Path;

use worth_store_physical_format::{
    decode_inline_record, DurableInlineRecordPlacement, PersistedRecordIdentity,
};
use worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff;

const PAGE_BYTES: usize = 16 * 1024;

pub fn store_identity(marker: &Path, record: PersistedRecordIdentity) {
    let mut bytes = Vec::with_capacity(24);
    bytes.extend_from_slice(&record.allocation_epoch());
    bytes.extend_from_slice(&record.ordinal().to_le_bytes());
    std::fs::write(marker.join("record-id.bin"), bytes).unwrap();
}

pub fn load_identity(marker: &Path) -> PersistedRecordIdentity {
    let bytes = std::fs::read(marker.join("record-id.bin")).unwrap();
    let mut epoch = [0; 16];
    epoch.copy_from_slice(&bytes[..16]);
    let ordinal = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    PersistedRecordIdentity::new(epoch, ordinal).unwrap()
}

pub fn selected_payload(
    root: &Path,
    handoff: &RecoveredPhysicalRuntimeHandoff,
    record: PersistedRecordIdentity,
) -> Vec<u8> {
    let placement = handoff
        .selected_sources()
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.record() == record =>
            {
                Some(*inline)
            }
            _ => None,
        })
        .expect("selected root does not contain the producer record");
    inline_payload(root, placement)
}

pub fn root_generation(handoff: &RecoveredPhysicalRuntimeHandoff) -> u64 {
    handoff
        .selected_sources()
        .root()
        .selected()
        .selector()
        .root_generation()
}

fn inline_payload(root: &Path, placement: DurableInlineRecordPlacement) -> Vec<u8> {
    let name = format!(
        "segment-{:016x}-{:016x}.pages",
        placement.segment().get(),
        placement.segment_generation()
    );
    let bytes = std::fs::read(root.join("families/records/segments").join(name))
        .expect("selected segment file");
    for page in bytes.chunks(PAGE_BYTES) {
        if let Ok((range, _)) = decode_inline_record(
            page,
            placement.record(),
            placement.page_cell(),
            placement.slot_cell(),
        ) {
            return page[range.range()].to_vec();
        }
    }
    panic!("selected placement bytes are not in its segment generation");
}
