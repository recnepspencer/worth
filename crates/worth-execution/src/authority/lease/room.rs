//! The one memory check every charge on a lease takes: the room each limit
//! on the charge's path leaves it, and the limit that refuses it.

use super::{LeaseNode, Ledger};
use crate::authority::{MemoryLimitDenial, MemoryLimitLevel};

/// Checks `requested` bytes against every limit on a charge's path,
/// innermost first: each lease on `lineage` from the charge's own outward,
/// then the process. Each lineage entry carries what the charge already holds
/// on that lease, and `released` what it holds on the process; both count as
/// room. The first limit short of room refuses, naming itself and its room.
pub(super) fn check<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    released: u64,
    lineage: impl IntoIterator<Item = (&'a LeaseNode, u64)> + 'a,
    requested: u64,
) -> Result<(), MemoryLimitDenial> {
    rooms(ledger, process_limit, released, lineage).try_for_each(|(admitted, level)| {
        if requested > admitted {
            Err(MemoryLimitDenial::refused(requested, admitted, level))
        } else {
            Ok(())
        }
    })
}

pub(super) fn available<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    lineage: impl IntoIterator<Item = &'a LeaseNode> + 'a,
) -> u64 {
    rooms(
        ledger,
        process_limit,
        0,
        lineage.into_iter().map(|node| (node, 0)),
    )
    .map(|(admitted, _)| admitted)
    .min()
    .unwrap_or(0)
}

fn rooms<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    released: u64,
    lineage: impl IntoIterator<Item = (&'a LeaseNode, u64)> + 'a,
) -> impl Iterator<Item = (u64, MemoryLimitLevel)> + 'a {
    (0..)
        .zip(lineage)
        .map(|(ancestor, (node, held))| {
            let used = ledger
                .nodes
                .get(&node.id)
                .map_or(0, |usage| usage.charged_memory_bytes);
            (
                node.charged_memory_bytes.saturating_sub(used - held),
                MemoryLimitLevel::Policy { ancestor },
            )
        })
        .chain(std::iter::once((
            process_limit
                .unwrap_or(u64::MAX)
                .saturating_sub(ledger.charged_memory_bytes - released),
            MemoryLimitLevel::Process,
        )))
}
