//! Semantic class inventory from independently authenticated C.5 route leaves.
//! Schema-2 routes are explicitly unknown, not inferred from mutable bytes.

use std::collections::BTreeMap;

use super::{ChildExpectation, ChildScope, TraversalOrigin};
use worth_foundational::PhysicalArtifactFamily as Family;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RouteClass {
    UnknownLegacy,
    Opaque,
    Blob(u8),
    DerivedDirectory,
    BTreeNode,
}

#[derive(Default)]
pub(crate) struct RouteInventory {
    pub(crate) classes: BTreeMap<[u8; 24], RouteClass>,
    pub(crate) intact: bool,
    extent_tiers: Vec<(u64, u8)>,
    header_tier_epoch: Option<(Option<u64>, u64)>,
}

impl RouteInventory {
    pub(crate) fn new() -> Self {
        Self {
            classes: BTreeMap::new(),
            intact: true,
            extent_tiers: Vec::new(),
            header_tier_epoch: None,
        }
    }

    pub(crate) fn note_failure(&mut self) {
        self.intact = false;
    }

    pub(crate) fn note_free_space_header(&mut self, bytes: &[u8]) {
        let Some(payload) = bytes.get(48..) else {
            self.intact = false;
            return;
        };
        let next_arena =
            u64::from_le_bytes(payload[144..152].try_into().expect("validated header"));
        let epoch = (payload.len() == 176).then(|| {
            u64::from_le_bytes(payload[168..176].try_into().expect("validated tier epoch"))
        });
        if self
            .header_tier_epoch
            .replace((epoch, next_arena))
            .is_some()
        {
            self.intact = false;
        }
    }

    pub(crate) fn tier_intact(&self) -> bool {
        let Some((epoch, next_arena)) = self.header_tier_epoch else {
            return false;
        };
        self.extent_tiers.iter().all(|(arena, observed)| {
            if *arena == 0 || *arena >= next_arena {
                return false;
            }
            let expected = match epoch.filter(|epoch| *arena >= *epoch) {
                None => 0,
                Some(epoch) => ((arena - epoch) % 3) as u8,
            };
            *observed == expected
        })
    }

    pub(crate) fn tier_mismatch(&self) -> bool {
        self.header_tier_epoch.is_some() && !self.tier_intact()
    }

    pub(crate) fn tier_epoch_start(&self) -> Option<u64> {
        self.header_tier_epoch.and_then(|(epoch, _)| epoch)
    }

    pub(crate) fn observe(&mut self, expected: &ChildExpectation, bytes: &[u8]) {
        if !matches!(expected.scope, ChildScope::Tree { level: 0, .. }) {
            return;
        }
        let Some(payload) = bytes.get(48..) else {
            self.intact = false;
            return;
        };
        let Some(body) = payload.get(40..) else {
            self.intact = false;
            return;
        };
        if body.len() % 88 != 0 {
            self.intact = false;
            return;
        }
        for entry in body.chunks_exact(88) {
            let Some(record) = entry.get(..24).and_then(|bytes| bytes.try_into().ok()) else {
                self.intact = false;
                return;
            };
            let class = if bytes[9] == 2 {
                RouteClass::UnknownLegacy
            } else {
                match entry[25] {
                    0 => RouteClass::UnknownLegacy,
                    1 => RouteClass::Opaque,
                    2 => RouteClass::Blob(entry[26]),
                    3 => RouteClass::BTreeNode,
                    4 => RouteClass::DerivedDirectory,
                    _ => {
                        self.intact = false;
                        return;
                    }
                }
            };
            if self.classes.insert(record, class).is_some() {
                self.intact = false;
            }
            let tier = if bytes[9] == 2 { 0 } else { entry[29] };
            match entry[24] {
                1 if tier != 0 => self.intact = false,
                2 => self.extent_tiers.push((
                    u64::from_le_bytes(entry[32..40].try_into().expect("validated route")),
                    tier,
                )),
                _ => {}
            }
        }
    }
}

pub(super) fn note_route_failure(
    origin: TraversalOrigin,
    generation: u64,
    current: Option<u64>,
    expected: &ChildExpectation,
    selected: &mut RouteInventory,
    historical: &mut RouteInventory,
) {
    if origin != TraversalOrigin::RootManifest || expected.family != Family::RootRoutingBlock {
        return;
    }
    if Some(generation) == current {
        selected.note_failure();
    }
    if current.and_then(|value| value.checked_sub(1)) == Some(generation) {
        historical.note_failure();
    }
}

#[cfg(test)]
mod tests {
    use super::RouteInventory;

    #[test]
    fn selected_arena_tier_must_follow_durable_epoch() {
        let mut routes = RouteInventory::new();
        routes.header_tier_epoch = Some((Some(7), 11));
        routes
            .extent_tiers
            .extend([(6, 0), (7, 0), (8, 1), (9, 2), (10, 0)]);
        assert!(routes.tier_intact());
        routes.extent_tiers[2].1 = 2;
        assert!(routes.tier_mismatch());
        routes.extent_tiers[2].1 = 1;
        routes.extent_tiers.push((11, 1));
        assert!(routes.tier_mismatch());

        let mut legacy = RouteInventory::new();
        legacy.header_tier_epoch = Some((None, 11));
        legacy.extent_tiers.push((9, 0));
        assert!(legacy.tier_intact());
        legacy.extent_tiers[0].1 = 1;
        assert!(legacy.tier_mismatch());
    }
}
