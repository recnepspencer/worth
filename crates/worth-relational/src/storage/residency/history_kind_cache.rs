//! Check a history member's supplied caches against its authenticated projection.
use super::region_image::{RegionAdjacencyKinds, RegionHistoryImage};
use crate::identity::data::{KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn verify(image: &RegionHistoryImage) -> Result<(), Cause> {
    if image
        .kind_projection
        .windows(2)
        .any(|pair| pair[0].relation >= pair[1].relation)
    {
        return Err(Cause::Damaged);
    }
    for (incoming, caches) in [
        (false, &image.adjacency_kinds),
        (true, &image.reverse_adjacency_kinds),
    ] {
        let mut expected = BTreeMap::<u64, BTreeMap<KindId, BTreeSet<RelationId>>>::new();
        for claim in &image.kind_projection {
            let slots = if incoming {
                &claim.incoming
            } else {
                &claim.outgoing
            };
            for &slot in slots {
                expected
                    .entry(slot as u64)
                    .or_default()
                    .entry(claim.kind)
                    .or_default()
                    .insert(claim.relation);
            }
        }
        verify_direction(caches, expected)?;
    }
    Ok(())
}

fn verify_direction(
    caches: &[RegionAdjacencyKinds],
    mut expected: BTreeMap<u64, BTreeMap<KindId, BTreeSet<RelationId>>>,
) -> Result<(), Cause> {
    if caches.windows(2).any(|pair| pair[0].slot >= pair[1].slot) {
        return Err(Cause::Damaged);
    }
    for cache in caches {
        if cache.kinds.windows(2).any(|pair| pair[0].0 >= pair[1].0)
            || cache
                .kinds
                .iter()
                .any(|(_, ids)| ids.is_empty() || ids.windows(2).any(|pair| pair[0] >= pair[1]))
        {
            return Err(Cause::Damaged);
        }
        let claims = expected.remove(&cache.slot).unwrap_or_default();
        if !cache
            .kinds
            .iter()
            .map(|(kind, ids)| (*kind, ids.iter().copied().collect::<BTreeSet<_>>()))
            .eq(claims)
        {
            return Err(Cause::IntegrityMismatch);
        }
    }
    if expected.is_empty() {
        Ok(())
    } else {
        Err(Cause::IntegrityMismatch)
    }
}
