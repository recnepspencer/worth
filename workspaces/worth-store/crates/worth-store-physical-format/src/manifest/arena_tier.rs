use crate::{ExtentArenaId, PhysicalTierClass};

/// The legacy arena namespace is entirely Primary. A tiered namespace starts
/// at a durable frontier; subsequent arena IDs cycle through three disjoint
/// classes without an unbounded per-arena manifest.
pub fn arena_tier_at_epoch(epoch: Option<u64>, arena: ExtentArenaId) -> PhysicalTierClass {
    let Some(epoch) = epoch else {
        return PhysicalTierClass::Primary;
    };
    if arena.get() < epoch {
        return PhysicalTierClass::Primary;
    }
    match (arena.get() - epoch) % 3 {
        0 => PhysicalTierClass::Primary,
        1 => PhysicalTierClass::Hot,
        _ => PhysicalTierClass::Cold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_arena_ids_remain_primary_and_new_ids_are_disjoint() {
        let id = |value| ExtentArenaId::new(value).unwrap();
        assert_eq!(
            arena_tier_at_epoch(None, id(99)),
            PhysicalTierClass::Primary
        );
        assert_eq!(
            arena_tier_at_epoch(Some(7), id(6)),
            PhysicalTierClass::Primary
        );
        assert_eq!(
            arena_tier_at_epoch(Some(7), id(7)),
            PhysicalTierClass::Primary
        );
        assert_eq!(arena_tier_at_epoch(Some(7), id(8)), PhysicalTierClass::Hot);
        assert_eq!(arena_tier_at_epoch(Some(7), id(9)), PhysicalTierClass::Cold);
        assert_eq!(
            arena_tier_at_epoch(Some(7), id(10)),
            PhysicalTierClass::Primary
        );
    }
}
