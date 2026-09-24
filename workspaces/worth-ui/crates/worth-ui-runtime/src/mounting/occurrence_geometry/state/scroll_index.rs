//! Immutable layout membership: one preorder plus descendant ranges, and exact
//! region slots. Rebuilt at structural admission/retirement, never at a tick.
use super::*;

type RegionSlot = (worth_ui_dsl::UiMosaicRegionDeclarationIdentity, usize);

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiScrollGeometryIndex {
    order: Vec<UiMountedInstanceIdentity>,
    descendants: BTreeMap<UiMountedInstanceIdentity, std::ops::Range<usize>>,
    regions: BTreeMap<UiMountedInstanceIdentity, Vec<RegionSlot>>,
}

impl UiScrollGeometryIndex {
    fn reserved_bytes(&self) -> Option<usize> {
        use std::mem::size_of;
        let node_links = 6 * size_of::<usize>();
        let mut bytes = size_of::<Self>()
            .checked_add(2 * size_of::<usize>())?
            .checked_add(
                self.order
                    .capacity()
                    .checked_mul(size_of::<UiMountedInstanceIdentity>())?,
            )?
            .checked_add(self.descendants.len().checked_mul(
                size_of::<(UiMountedInstanceIdentity, std::ops::Range<usize>)>() + node_links,
            )?)?
            .checked_add(self.regions.len().checked_mul(
                size_of::<(UiMountedInstanceIdentity, Vec<RegionSlot>)>() + node_links,
            )?)?;
        for slots in self.regions.values() {
            bytes = bytes.checked_add(slots.capacity().checked_mul(size_of::<RegionSlot>())?)?;
        }
        Some(bytes)
    }

    pub(super) fn build(geometry: &UiMountedSurfaceGeometry) -> Self {
        let mut index = Self::default();
        let mut pending = geometry
            .occurrences
            .iter()
            .filter(|(_, row)| row.parent.is_none())
            .map(|(instance, _)| (*instance, None))
            .collect::<Vec<_>>();
        while let Some((instance, start)) = pending.pop() {
            if let Some(start) = start {
                index.descendants.insert(instance, start..index.order.len());
            } else {
                index.order.push(instance);
                pending.push((instance, Some(index.order.len())));
                if let Some(children) = geometry.children.get(&instance) {
                    pending.extend(children.iter().rev().map(|child| (*child, None)));
                }
            }
        }
        for (declaration, rows) in &geometry.regions {
            for (slot, (owner, _, _)) in rows.iter().enumerate() {
                index
                    .regions
                    .entry(*owner)
                    .or_default()
                    .push((*declaration, slot));
            }
        }
        index
    }

    pub(super) fn descendants(
        &self,
        owner: UiMountedInstanceIdentity,
    ) -> &[UiMountedInstanceIdentity] {
        self.descendants
            .get(&owner)
            .map_or(&[], |range| &self.order[range.clone()])
    }

    pub(super) fn regions(&self, owner: UiMountedInstanceIdentity) -> &[RegionSlot] {
        self.regions.get(&owner).map_or(&[], Vec::as_slice)
    }
}

type UiScrollReservations = std::rc::Rc<BTreeMap<UiSemanticSurfaceIdentity, usize>>;

/// The last reservation snapshot handed out. Every read recomputes each value
/// and reuses this one only when all of them match, so it never answers for
/// geometry it did not see; it spares an unchanged turn an identical map.
#[derive(Clone, Debug, Default)]
pub(super) struct UiScrollReservationMemo(std::cell::RefCell<UiScrollReservations>);

impl PartialEq for UiScrollReservationMemo {
    /// A memo of `surfaces`, which the owning state already compares.
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl UiMountedOccurrenceGeometryState {
    pub(crate) fn scroll_geometry_reservations(&self) -> Option<UiScrollReservations> {
        let mut memo = self.reservations.0.borrow_mut();
        let mut remembered = memo.iter();
        let mut unchanged = memo.len() == self.surfaces.len();
        for (surface, geometry) in &self.surfaces {
            let bytes = geometry.scroll_index.reserved_bytes()?;
            unchanged &= remembered.next() == Some((surface, &bytes));
        }
        if !unchanged {
            *memo = std::rc::Rc::new(
                self.surfaces
                    .iter()
                    .map(|(surface, geometry)| {
                        Some((*surface, geometry.scroll_index.reserved_bytes()?))
                    })
                    .collect::<Option<_>>()?,
            );
        }
        Some(std::rc::Rc::clone(&memo))
    }
}

impl UiMountedSurfaceGeometry {
    pub(super) fn index_scroll_geometry(mut self) -> Self {
        self.scroll_index = std::sync::Arc::new(UiScrollGeometryIndex::build(&self));
        self
    }
}
