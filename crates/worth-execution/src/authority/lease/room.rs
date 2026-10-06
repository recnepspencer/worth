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
    ledger: &Ledger,
    process_limit: u64,
    released: u64,
    lineage: impl IntoIterator<Item = (&'a LeaseNode, u64)>,
    requested: u64,
) -> Result<(), MemoryLimitDenial> {
    let refuse = |admitted: u64, level| {
        if requested > admitted {
            Err(MemoryLimitDenial {
                requested,
                admitted,
                level,
            })
        } else {
            Ok(())
        }
    };
    for (ancestor, (node, held)) in (0..).zip(lineage) {
        let used = ledger
            .nodes
            .get(&node.id)
            .map_or(0, |usage| usage.charged_memory_bytes);
        refuse(
            node.charged_memory_bytes.saturating_sub(used - held),
            MemoryLimitLevel::Policy { ancestor },
        )?;
    }
    refuse(
        process_limit.saturating_sub(ledger.charged_memory_bytes - released),
        MemoryLimitLevel::Process,
    )
}
