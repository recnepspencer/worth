//! A selected rewrite is record-preserving, but its WAL member still needs
//! exact operation, group, LSN, and root-lineage binding before NoRelease can
//! exclude it from the canonical record-drop path.

use worth_store_physical_format::{PhysicalRewriteRedo, PhysicalRewriteRedoDenial};
use worth_store_wal::WalLsnRange;

use super::Denial;

pub(super) fn admit_selected_rewrite(
    encoded: &[u8],
    member_operation: [u8; 32],
    member_group: [u8; 32],
    member_lsn: WalLsnRange,
    operations: impl Iterator<Item = ([u8; 32], [u8; 32])>,
    selected_root_generation: u64,
) -> Result<bool, Denial> {
    let rewrite = match PhysicalRewriteRedo::decode(encoded, u64::MAX) {
        Ok(rewrite) => rewrite,
        Err(PhysicalRewriteRedoDenial::WrongDomain) => return Ok(false),
        Err(_) => return Err(Denial::WalFate),
    };
    let mut fingerprint = None;
    for (identity, request_fingerprint) in operations {
        if identity == member_operation && fingerprint.replace(request_fingerprint).is_some() {
            return Err(Denial::WalFate);
        }
    }
    let fingerprint = fingerprint.ok_or(Denial::WalFate)?;
    let start = member_lsn.start().get();
    if member_lsn.end_exclusive().get() != start.checked_add(1).ok_or(Denial::WalFate)?
        || rewrite.operation() != fingerprint
        || rewrite.group() != member_group
        || rewrite.page_lsn() != start
        || rewrite.source_root_generation() == 0
        || rewrite.source_generation() == 0
        || rewrite.source_root_generation().checked_add(1)
            != Some(rewrite.resulting_root_generation())
        || rewrite.source_generation().checked_add(1) != Some(rewrite.destination_generation())
        || rewrite.resulting_root_generation() > selected_root_generation
    {
        return Err(Denial::WalFate);
    }
    Ok(true)
}

#[cfg(test)]
#[path = "rewrite/tests.rs"]
mod tests;
