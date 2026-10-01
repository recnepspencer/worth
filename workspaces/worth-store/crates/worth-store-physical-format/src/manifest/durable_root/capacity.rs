use super::{CURRENT_ROOT_MANIFEST_ENTRY_BYTES, CURRENT_ROOT_MANIFEST_PREFIX_BYTES};
use crate::PhysicalRecordFormatDeclaration;

pub const fn maximum_current_root_entries(format: PhysicalRecordFormatDeclaration) -> u16 {
    let available = format.page_size().bytes() as usize
        - crate::record_framing::DURABLE_FRAME_HEADER_BYTES
        - CURRENT_ROOT_MANIFEST_PREFIX_BYTES;
    let entries = available / CURRENT_ROOT_MANIFEST_ENTRY_BYTES;
    if entries > u16::MAX as usize {
        u16::MAX
    } else {
        entries as u16
    }
}
