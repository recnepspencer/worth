//! Canonical WAL v1 byte grammar and checksum mechanism.
//!
//! This module interprets and constructs bytes. It does not perform file I/O,
//! establish prefix continuity, bind Store authority, or select recovery policy.

mod bounded_decode;
mod checksum;
mod encode;
mod header;
mod identity;

pub use checksum::{
    wal_frame_v1_declared_identity_digest, wal_frame_v1_validation_digest,
    WalFrameV1CalculatedChecksums, WalFrameV1ChecksumCalculator,
};
pub use encode::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
pub use header::{decode_wal_frame_v1_header, WalFrameV1Denial, WalFrameV1Header};
pub use identity::WalSegmentIdentity;

pub use crate::integrity_declarations::families::{
    WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES, WAL_FRAME_V1_VERSION,
};
pub use bounded_decode::{
    decode_bounded_wal_frame_v1, BoundedWalFrameV1, WalFrameV1BoundedDecodeDenial,
    WalFrameV1ChecksumMismatch,
};
