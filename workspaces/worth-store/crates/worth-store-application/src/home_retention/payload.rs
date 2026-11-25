//! The fixed owner bytes recorded with each durable frontier pin.
use worth_foundational::facade::HomeIncarnation;
use worth_runtime_world::facade::PinScope;

const ENCODED_BYTES: usize = 8 + 32 + 8;

pub(super) struct RecordedFrontierPin {
    pub(super) incarnation: HomeIncarnation,
    pub(super) program_digest: [u8; 32],
}

pub(super) fn encode(
    incarnation: HomeIncarnation,
    program_digest: [u8; 32],
    scope: PinScope,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(ENCODED_BYTES);
    bytes.extend_from_slice(&incarnation.get().to_le_bytes());
    bytes.extend_from_slice(&program_digest);
    bytes.extend_from_slice(&scope.get().to_le_bytes());
    bytes
}

pub(super) fn decode(bytes: &[u8]) -> Option<RecordedFrontierPin> {
    if bytes.len() != ENCODED_BYTES {
        return None;
    }
    Some(RecordedFrontierPin {
        incarnation: HomeIncarnation::from_recorded(u64::from_le_bytes(
            bytes[..8].try_into().ok()?,
        ))?,
        program_digest: bytes[8..40].try_into().ok()?,
    })
}
