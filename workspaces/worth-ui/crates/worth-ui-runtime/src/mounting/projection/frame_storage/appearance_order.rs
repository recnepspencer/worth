use crate::mounting::spatial_index::UiMountedSpatialWork;
use crate::runtime::persistent_index::UiPersistentOrdMap;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

mod partition;
mod row;
use partition::Surface;
use row::Row;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceOrderDenial {
    MountedGeometryUnavailable,
    InvalidGeometry,
    QueryBudgetExceeded,
    CapacityExceeded,
    SurfaceCapacityExceeded,
    MechanicCapacityExceeded,
    Ambiguous {
        surface: UiSemanticSurfaceIdentity,
        first: UiMountedInstanceIdentity,
        second: UiMountedInstanceIdentity,
        rank: u32,
    },
}

/// Derived acceleration over completed appearance mechanics. It neither issues
/// layout bounds nor chooses a tie-break. The frame candidate owns its lifetime.
#[derive(Clone, Default)]
pub(super) struct UiMountedAppearanceOrderIndex {
    surfaces: UiPersistentOrdMap<UiSemanticSurfaceIdentity, Surface>,
    node_count: usize,
    surface_bytes: usize,
}

#[derive(Clone)]
struct Update {
    surface: UiSemanticSurfaceIdentity,
    instance: UiMountedInstanceIdentity,
    rows: Vec<Row>,
}

impl UiMountedAppearanceOrderIndex {
    pub(super) fn forget_surface(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) -> UiMountedSpatialWork {
        let mut work = UiMountedSpatialWork::default();
        let (previous, probes) = self.surfaces.get_with_probes(&surface);
        work.map_key_probes += probes;
        if let Some(previous) = previous {
            self.node_count -= previous.len();
            self.surface_bytes -= previous.retained_bytes();
        }
        record_map(&mut work, self.surfaces.remove_with_work(&surface).1);
        work
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.surfaces
            .retained_structural_bytes()
            .expect("bounded appearance surfaces")
            + self.surface_bytes
    }

    pub(super) fn admit(
        &mut self,
        frame: &super::UiMountedProjectionFrame,
        nodes: &[super::appearance_output::UiMountedAppearanceNodeWork],
    ) -> (
        Result<(), UiMountedAppearanceOrderDenial>,
        UiMountedSpatialWork,
    ) {
        let mut work = UiMountedSpatialWork::default();
        let result = nodes
            .iter()
            .map(|node| row::derive(frame, node, &mut work))
            .collect::<Result<Vec<_>, _>>()
            .and_then(|updates| self.apply(&updates, &mut work));
        (result, work)
    }

    fn apply(
        &mut self,
        updates: &[Update],
        work: &mut UiMountedSpatialWork,
    ) -> Result<(), UiMountedAppearanceOrderDenial> {
        if updates.iter().any(|update| update.rows.len() > 2) {
            return Err(UiMountedAppearanceOrderDenial::MechanicCapacityExceeded);
        }
        let updates = updates
            .iter()
            .filter(|update| {
                let (surface, probes) = self.surfaces.get_with_probes(&update.surface);
                work.map_key_probes += probes;
                !surface.map_or(update.rows.is_empty(), |surface| {
                    surface.matches(update, work)
                })
            })
            .collect::<Vec<_>>();
        let mut candidate = self.clone();
        // Removing the entire departing set first permits simultaneous rank swaps
        // and replacements whose old boxes overlap another arriving box.
        for update in &updates {
            candidate.change_surface(update, false, work)?;
        }
        for update in &updates {
            candidate.change_surface(update, true, work)?;
            if candidate.node_count > super::appearance_state::APPEARANCE_STATE_CAPACITY {
                return Err(UiMountedAppearanceOrderDenial::CapacityExceeded);
            }
            if candidate.surfaces.len() > 64 {
                return Err(UiMountedAppearanceOrderDenial::SurfaceCapacityExceeded);
            }
        }
        *self = candidate;
        Ok(())
    }

    fn change_surface(
        &mut self,
        update: &Update,
        insert: bool,
        work: &mut UiMountedSpatialWork,
    ) -> Result<(), UiMountedAppearanceOrderDenial> {
        let (surface, probes) = self.surfaces.get_with_probes(&update.surface);
        work.map_key_probes += probes;
        let mut surface = surface.cloned().unwrap_or_default();
        let previous_count = surface.len();
        let previous_bytes = surface.retained_bytes();
        if insert {
            surface.insert(update, work)?;
        } else {
            surface.remove(update.instance, work);
        }
        self.node_count = self.node_count - previous_count + surface.len();
        self.surface_bytes = self.surface_bytes - previous_bytes + surface.retained_bytes();
        let mutation = if surface.is_empty() {
            self.surfaces.remove_with_work(&update.surface).1
        } else {
            self.surfaces.insert_with_work(update.surface, surface)
        };
        record_map(work, mutation);
        Ok(())
    }
}

fn record_map(
    work: &mut UiMountedSpatialWork,
    mutation: crate::runtime::persistent_index::UiPersistentIndexMutationWork,
) {
    work.map_key_probes += mutation.key_probes();
    work.map_node_copies += mutation.node_copies();
}

#[cfg(test)]
mod tests;
