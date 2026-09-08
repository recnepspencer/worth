pub(crate) mod crc32c;
mod durable_frame;
#[cfg(test)]
mod durable_frame_canonicality;
mod framing;

pub use durable_frame::{
    decode_data_frame_page_lsn, encode_data_frame_page_lsn, DurableFrameDenial, DurableFrameKind,
    PhysicalPageLsn, FRAME_HEADER_BYTES as DURABLE_FRAME_HEADER_BYTES,
};
pub(crate) use durable_frame::{
    decode_frame as decode_durable_frame, encode_frame as encode_durable_frame,
    initialize_frame as initialize_durable_frame,
    initialize_frame_reusing as initialize_durable_frame_reusing,
    reseal_frame as reseal_durable_frame,
};

pub fn durable_artifact_checksum(bytes: &[u8]) -> u32 {
    crc32c::checksum(&[bytes])
}
pub use framing::*;
