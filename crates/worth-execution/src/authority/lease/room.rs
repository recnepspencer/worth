//! One locked memory admission across the request lineage and process.
use super::{LeaseNode, Ledger};
use crate::authority::{MemoryLimitDenial, MemoryLimitLevel};

pub(super) fn check<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    released: u64,
    lineage: impl IntoIterator<Item = (&'a LeaseNode, u64)> + 'a,
    requested: u64,
) -> Result<(), MemoryLimitDenial> {
    for (ceiling, charged, level) in charges(ledger, process_limit, released, lineage) {
        if let Some(ceiling) = ceiling {
            let admitted = ceiling.saturating_sub(charged);
            if requested > admitted {
                return Err(MemoryLimitDenial::refused(requested, admitted, level));
            }
        }
        charged
            .checked_add(requested)
            .ok_or_else(|| MemoryLimitDenial::overflow(requested, charged, level))?;
    }
    Ok(())
}

/// None means no supplied ceiling on this lineage; counters are still checked on admission.
pub(super) fn available<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    lineage: impl IntoIterator<Item = &'a LeaseNode> + 'a,
) -> Option<u64> {
    charges(
        ledger,
        process_limit,
        0,
        lineage.into_iter().map(|node| (node, 0)),
    )
    .filter_map(|(ceiling, charged, _)| ceiling.map(|cap| cap.saturating_sub(charged)))
    .min()
}

fn charges<'a>(
    ledger: &'a Ledger,
    process_limit: Option<u64>,
    released: u64,
    lineage: impl IntoIterator<Item = (&'a LeaseNode, u64)> + 'a,
) -> impl Iterator<Item = (Option<u64>, u64, MemoryLimitLevel)> + 'a {
    (0..)
        .zip(lineage)
        .map(|(ancestor, (node, held))| {
            let used = ledger
                .nodes
                .get(&node.id)
                .map_or(0, |usage| usage.charged_memory_bytes);
            (
                node.charged_memory_bytes,
                used - held,
                MemoryLimitLevel::Policy { ancestor },
            )
        })
        .chain(std::iter::once((
            process_limit,
            ledger.charged_memory_bytes - released,
            MemoryLimitLevel::Process,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_process_ceiling_keeps_counter_overflow_distinct_from_capacity() {
        let ledger = Ledger {
            charged_memory_bytes: u64::MAX,
            ..Ledger::default()
        };
        assert_eq!(
            check(&ledger, None, 0, std::iter::empty(), 1),
            Err(MemoryLimitDenial::CounterOverflow {
                requested: 1,
                charged: u64::MAX,
                level: MemoryLimitLevel::Process,
            })
        );
        assert_eq!(available(&ledger, None, std::iter::empty()), None);
        assert_eq!(
            check(&ledger, Some(u64::MAX), 0, std::iter::empty(), 1),
            Err(MemoryLimitDenial::Capacity {
                requested: 1,
                admitted: 0,
                level: MemoryLimitLevel::Process,
            })
        );
        assert_eq!(ledger.charged_memory_bytes, u64::MAX);
        // Only bytes genuinely released by this reservation count as room.
        assert_eq!(check(&ledger, None, 1, std::iter::empty(), 1), Ok(()));
    }
}
