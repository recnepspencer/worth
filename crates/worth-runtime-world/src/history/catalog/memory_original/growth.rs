//! Fixed-tier backing adoption rules for overlapping provisional reservations.
use super::Slot;
use worth_relational::facade::branch::RelationalRecordedComponentRefusal as Refusal;

pub(super) const MAX_GROWTH_TIERS: usize = usize::BITS as usize + 1;

#[derive(Debug)]
pub(super) struct ProvisionalGrowth {
    pub(super) backing: Vec<Slot>,
    pub(super) funding: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) fn reserve_provisional_growth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    current_cap: usize,
    needed: usize,
    malformed: impl Fn() -> Refusal,
) -> Result<(ProvisionalGrowth, u8), Refusal> {
    let (tier, capacity) = next_growth_tier(current_cap, needed).ok_or_else(&malformed)?;
    let bytes = capacity
        .checked_mul(std::mem::size_of::<Slot>())
        .ok_or_else(&malformed)?;
    let funding = execution
        .reserve_retained_memory(bytes as u64)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut charged = None;
    let scope = execution.run_native_with_checkpoint(|context| {
        charged = Some(if context.has_accounting() {
            context.checkpoint(capacity as u64)
        } else {
            context.poll()
        });
    });
    match (scope, charged) {
        (Ok(_), Some(Ok(()))) => {}
        (_, Some(Err(cause))) => return Err(Refusal::Request(cause.into())),
        (Err(cause), _) => return Err(Refusal::Request(cause.into())),
        (Ok(_), None) => unreachable!("registry growth charged before allocation"),
    }
    let mut backing = Vec::new();
    backing
        .try_reserve_exact(capacity)
        .map_err(|_| malformed())?;
    // Vec reports the requested element capacity; refuse a wider backing
    // before retaining it against this request's exact capacity quote.
    if backing.capacity() != capacity {
        return Err(malformed());
    }
    Ok((ProvisionalGrowth { backing, funding }, tier))
}

pub(super) const fn tier_capacity(tier: u8) -> usize {
    if tier == 0 {
        0
    } else if (tier as u32) >= usize::BITS {
        usize::MAX
    } else {
        (1usize << tier) - 1
    }
}

pub(super) fn next_growth_tier(current_cap: usize, needed: usize) -> Option<(u8, usize)> {
    let mut tier: u8 = 1;
    while (tier as usize) < MAX_GROWTH_TIERS {
        let cap = tier_capacity(tier);
        if cap >= needed && cap > current_cap {
            return Some((tier, cap));
        }
        tier += 1;
    }
    None
}

/// True iff dropping a sealing slot's provisional `growth_cap` backing would
/// leave either installed `cur_cap` or some intermediate funded tier unable to
/// hold `used_after` plus every remaining pending reservation at or below that
/// tier's capacity.
pub(super) fn backing_needed_for_pending(
    pending_by_tier: &[usize; MAX_GROWTH_TIERS],
    cur_cap: usize,
    growth_cap: usize,
    used_after: usize,
) -> bool {
    let mut dependent = pending_by_tier[0];
    let mut tier: u8 = 1;
    while (tier as usize) < MAX_GROWTH_TIERS {
        let cap = tier_capacity(tier);
        if cap > cur_cap {
            break;
        }
        dependent = dependent.saturating_add(pending_by_tier[tier as usize]);
        tier += 1;
    }
    if used_after.saturating_add(dependent) > cur_cap {
        return true;
    }
    while (tier as usize) < MAX_GROWTH_TIERS {
        let cap = tier_capacity(tier);
        if cap >= growth_cap {
            break;
        }
        dependent = dependent.saturating_add(pending_by_tier[tier as usize]);
        if used_after.saturating_add(dependent) > cap {
            return true;
        }
        tier += 1;
    }
    false
}
