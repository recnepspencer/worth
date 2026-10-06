use sha2::{Digest, Sha256};

/// SHA-256 of one complete durable root manifest frame: its exact frame
/// identity.
pub fn durable_root_manifest_frame_digest(frame: &[u8]) -> [u8; 32] {
    Sha256::digest(frame).into()
}
