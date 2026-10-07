use super::free_space_tree::ArenaGeometry;
use super::observation::{
    OfflineAllocationClass, OfflineFreeSpaceMembership, OfflineRecordPlacement,
};
use super::OfflineDurableManifestDenial;

/// Independent range accounting for this root. Gaps can be displaced or
/// reserved ranges; only the full artifact observer classifies those residues.
pub(super) fn validate(
    placements: &[OfflineRecordPlacement],
    free: &[OfflineFreeSpaceMembership],
    geometry: ArenaGeometry,
) -> Result<(), OfflineDurableManifestDenial> {
    let routed = placements.iter().filter_map(|placement| match *placement {
        OfflineRecordPlacement::Extent {
            arena,
            arena_offset,
            arena_length,
            ..
        } => Some((arena, arena_offset, arena_length)),
        _ => None,
    });
    let free = free
        .iter()
        .filter(|entry| entry.class == OfflineAllocationClass::ExtentArena)
        .map(|entry| {
            (
                entry.owner,
                entry.first_unallocated,
                entry.unallocated_count,
            )
        });
    let mut ranges = routed.chain(free).collect::<Vec<_>>();
    for &(arena, offset, length) in &ranges {
        if arena == 0
            || arena >= geometry.next_arena
            || length == 0
            || !offset.is_multiple_of(geometry.alignment)
            || !length.is_multiple_of(geometry.alignment)
            || offset
                .checked_add(length)
                .is_none_or(|end| end > geometry.capacity)
        {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
    }
    ranges.sort_unstable();
    if ranges
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0 && pair[0].1 + pair[0].2 > pair[1].1)
    {
        return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::observation::OfflineRecordIdentity;
    use super::*;

    fn route(arena: u64, offset: u64, length: u64) -> OfflineRecordPlacement {
        OfflineRecordPlacement::Extent {
            record: OfflineRecordIdentity::decode(&[1; 24]).unwrap(),
            extent: 1,
            generation: 1,
            payload_bytes: 3,
            arena,
            arena_offset: offset,
            arena_length: length,
        }
    }

    #[test]
    fn fully_occupied_arena_and_disjoint_routes_are_valid_but_overlap_is_not() {
        let geometry = ArenaGeometry {
            next_arena: 2,
            capacity: 65536,
            alignment: 4096,
        };
        assert_eq!(validate(&[route(1, 0, 65536)], &[], geometry), Ok(()));
        assert_eq!(
            validate(&[route(1, 0, 32768), route(1, 32768, 32768)], &[], geometry),
            Ok(())
        );
        assert!(validate(&[route(1, 0, 32768), route(1, 16384, 32768)], &[], geometry).is_err());
        assert!(validate(&[route(2, 0, 4096)], &[], geometry).is_err());
        assert!(validate(&[route(1, 61440, 8192)], &[], geometry).is_err());
        assert!(validate(&[route(1, 1, 4096)], &[], geometry).is_err());
        let free = OfflineFreeSpaceMembership {
            class: OfflineAllocationClass::ExtentArena,
            owner: 1,
            first_unallocated: 4096,
            unallocated_count: 4096,
            generation: 1,
        };
        assert!(validate(&[route(1, 0, 8192)], &[free], geometry).is_err());
        assert_eq!(validate(&[route(1, 0, 4096)], &[free], geometry), Ok(()));
    }
}
