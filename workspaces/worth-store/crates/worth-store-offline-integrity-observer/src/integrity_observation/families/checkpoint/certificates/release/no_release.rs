//! Independent, format-only interpretation of positive no-release custody.

use super::{valid_checkpoint, Cursor, PREFIX};

pub(super) const WIRE_BYTES: usize = PREFIX + 24 + 8 + 32 + 8 + 32 + 32;

#[derive(Clone)]
pub(crate) struct NoRelease {
    pub(crate) checkpoint: [u8; 24],
    pub(crate) root_generation: u64,
    pub(crate) root_sha: [u8; 32],
    #[cfg(test)]
    pub(crate) prior_sequence: u64,
    #[cfg(test)]
    pub(crate) prior_root_sha: [u8; 32],
    #[cfg(test)]
    pub(crate) prior_marker_payload_sha: [u8; 32],
}

pub(super) fn read(cursor: &mut Cursor<'_>) -> Option<NoRelease> {
    let checkpoint = cursor.take::<24>()?;
    let root_generation = cursor.u64()?;
    let root_sha = cursor.take::<32>()?;
    let prior_sequence = cursor.u64()?;
    let prior_root_sha = cursor.take::<32>()?;
    let prior_marker_payload_sha = cursor.take::<32>()?;
    let has_prior = prior_sequence != 0;
    if !valid_checkpoint(checkpoint)
        || root_generation == 0
        || root_sha == [0; 32]
        || has_prior != (prior_root_sha != [0; 32])
        || has_prior != (prior_marker_payload_sha != [0; 32])
        || (has_prior && prior_sequence >= u64::from_le_bytes(checkpoint[16..24].try_into().ok()?))
    {
        return None;
    }
    Some(NoRelease {
        checkpoint,
        root_generation,
        root_sha,
        #[cfg(test)]
        prior_sequence,
        #[cfg(test)]
        prior_root_sha,
        #[cfg(test)]
        prior_marker_payload_sha,
    })
}
