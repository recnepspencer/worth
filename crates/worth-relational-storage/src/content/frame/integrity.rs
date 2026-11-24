//! The sole production content-frame digest owner; every original byte is paid.
use super::super::{RelationalContentControl as Control, RelationalContentDenial as Denial};
use sha2::{Digest, Sha256};
use worth_foundational::facade::ContentRefId;

pub(super) fn digest(
    frame: &[u8],
    control: &mut Control<'_, '_, '_>,
) -> Result<ContentRefId, Denial> {
    const DOMAIN: &[u8] = b"worth.native.content.v1\0";
    let mut digest = Sha256::new();
    control.checkpoint(DOMAIN.len() as u64)?;
    digest.update(DOMAIN);
    for chunk in frame.chunks(4096) {
        control.checkpoint(chunk.len() as u64)?;
        digest.update(chunk);
    }
    Ok(ContentRefId(digest.finalize().into()))
}
