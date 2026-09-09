use crate::mounting::UiHitTestSpatialWork;
use worth_ui_host_contract::{
    UiMountedCoordinateSpace, UiMountedHitTestMechanic, UiMountedHitTestOrder,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use crate::mounting::spatial_index::{
    UiMountedSpatialBudget, UiMountedSpatialQuery, UiMountedSpatialQueryDenial,
    UiMountedSpatialTree,
};
use crate::mounting::UiMountedProjectionDenial;
use crate::runtime::persistent_index::{UiPersistentIndexMutationWork, UiPersistentOrdMap};

/// Canonical allocation-space hit rows with reconstructible order and spatial indexes.
/// Portal/Motion presentation geometry and current receipt admission remain downstream.
#[derive(Clone, Default)]
pub(in crate::mounting) struct UiMountedHitMechanicSource {
    rows: UiPersistentOrdMap<UiMountedInstanceIdentity, UiMountedHitTestMechanic>,
    order: UiPersistentOrdMap<
        (UiSemanticSurfaceIdentity, UiMountedHitTestOrder),
        UiMountedInstanceIdentity,
    >,
    spatial: UiPersistentOrdMap<(UiSurfaceBindingGeneration, u8), UiMountedSpatialTree>,
    spatial_node_bytes: usize,
    digest: u64,
}

impl UiMountedHitMechanicSource {
    pub(in crate::mounting::projection) fn replace(
        &mut self,
        instance: UiMountedInstanceIdentity,
        successor: Option<UiMountedHitTestMechanic>,
    ) -> Result<UiHitTestSpatialWork, UiMountedProjectionDenial> {
        let (previous, probes) = self.rows.get_with_probes(&instance);
        let previous = previous.copied();
        let mut work = UiHitTestSpatialWork {
            map_key_probes: probes,
            ..Default::default()
        };
        if let Some(row) = successor {
            if row.mounted_instance() != instance {
                return Err(UiMountedProjectionDenial::HitTestNodeReceiptMismatch);
            }
            let (owner, probes) = self.order.get_with_probes(&(row.surface(), row.order()));
            work.map_key_probes += probes;
            if owner.is_some_and(|owner| *owner != instance) {
                return Err(UiMountedProjectionDenial::DuplicateHitTestOrder {
                    surface: row.surface(),
                    order: row.order(),
                });
            }
        }
        // Row admission is complete. The enclosing completion admits final batch
        // capacity, allowing an insertion before a removal at the capacity boundary.
        self.update_spatial(previous, successor, &mut work);
        if let Some(row) = previous {
            self.digest ^= row_digest(row);
            record_map(
                &mut work,
                self.order.remove_with_work(&(row.surface(), row.order())).1,
            );
        }
        if let Some(row) = successor {
            self.digest ^= row_digest(row);
            record_map(
                &mut work,
                self.order
                    .insert_with_work((row.surface(), row.order()), instance),
            );
            record_map(&mut work, self.rows.insert_with_work(instance, row));
        } else if previous.is_some() {
            record_map(&mut work, self.rows.remove_with_work(&instance).1);
        }
        Ok(work)
    }

    pub(in crate::mounting) fn allocation_candidates(
        &self,
        binding: UiSurfaceBindingGeneration,
        space: UiMountedCoordinateSpace,
        point: [f64; 2],
        budget: UiMountedSpatialBudget,
    ) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
        if point.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(UiMountedSpatialQueryDenial::InvalidGeometry);
        }
        let (tree, probes) = self.spatial.get_with_probes(&(binding, space as u8));
        let result = tree.map_or_else(
            || {
                Ok(UiMountedSpatialQuery {
                    instances: Vec::new(),
                    work: Default::default(),
                })
            },
            |tree| tree.at_point(point, budget),
        );
        match result {
            Ok(mut query) => {
                query.work.map_key_probes += probes;
                Ok(query)
            }
            Err(UiMountedSpatialQueryDenial::NodeBudget { mut work }) => {
                work.map_key_probes += probes;
                Err(UiMountedSpatialQueryDenial::NodeBudget { work })
            }
            Err(UiMountedSpatialQueryDenial::CandidateBudget { mut work }) => {
                work.map_key_probes += probes;
                Err(UiMountedSpatialQueryDenial::CandidateBudget { work })
            }
            Err(denial) => Err(denial),
        }
    }

    pub(in crate::mounting) fn get(
        &self,
        instance: &UiMountedInstanceIdentity,
    ) -> Option<&UiMountedHitTestMechanic> {
        self.rows.get(instance)
    }

    pub(in crate::mounting) fn iter(
        &self,
    ) -> impl Iterator<Item = (&UiMountedInstanceIdentity, &UiMountedHitTestMechanic)> {
        self.rows.iter()
    }

    pub(in crate::mounting) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(in crate::mounting::projection) const fn digest(&self) -> u64 {
        self.digest
    }

    pub(in crate::mounting) fn retained_structural_bytes(&self) -> Option<usize> {
        self.rows
            .retained_structural_bytes()?
            .checked_add(self.order.retained_structural_bytes()?)?
            .checked_add(self.spatial.retained_structural_bytes()?)?
            .checked_add(self.spatial_node_bytes)
    }

    pub(in crate::mounting::projection) fn rebind(
        &self,
        replacements: &[(
            UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<(Self, UiHitTestSpatialWork), UiMountedProjectionDenial> {
        // Binding reconstruction is a cold whole-source lane; install only after admission.
        let mut rows = self.rows.iter().map(|(_, row)| *row).collect::<Vec<_>>();
        super::super::hit_test::rebind_hit_tests(&mut rows, replacements)?;
        let mut successor = Self::default();
        let mut work = UiHitTestSpatialWork {
            reconstructed_rows: rows.len(),
            ..Default::default()
        };
        for row in rows {
            work.merge(successor.replace(row.mounted_instance(), Some(row))?);
        }
        Ok((successor, work))
    }

    fn update_spatial(
        &mut self,
        previous: Option<UiMountedHitTestMechanic>,
        successor: Option<UiMountedHitTestMechanic>,
        work: &mut UiHitTestSpatialWork,
    ) {
        if let (Some(old), Some(new)) = (previous, successor) {
            if spatial_key(old) == spatial_key(new) {
                self.update_tree(
                    spatial_key(old),
                    old.mounted_instance(),
                    Some(region(old)),
                    Some(region(new)),
                    work,
                );
                return;
            }
        }
        if let Some(old) = previous {
            self.update_tree(
                spatial_key(old),
                old.mounted_instance(),
                Some(region(old)),
                None,
                work,
            );
        }
        if let Some(new) = successor {
            self.update_tree(
                spatial_key(new),
                new.mounted_instance(),
                None,
                Some(region(new)),
                work,
            );
        }
    }

    fn update_tree(
        &mut self,
        key: (UiSurfaceBindingGeneration, u8),
        instance: UiMountedInstanceIdentity,
        previous: Option<[f64; 4]>,
        successor: Option<[f64; 4]>,
        work: &mut UiHitTestSpatialWork,
    ) {
        if previous == successor {
            return;
        }
        let (tree, probes) = self.spatial.get_with_probes(&key);
        work.map_key_probes += probes;
        let mut tree = tree.cloned().unwrap_or_default();
        let previous_bytes = tree
            .retained_structural_bytes()
            .expect("retained tree bytes fit address space");
        let changed = tree
            .replace(instance, previous, successor)
            .expect("completed hit mechanics retain finite ordered geometry");
        work.merge(changed);
        if changed.node_visits == 0 && changed.node_copies == 0 {
            return;
        }
        self.spatial_node_bytes = self
            .spatial_node_bytes
            .checked_sub(previous_bytes)
            .and_then(|bytes| bytes.checked_add(tree.retained_structural_bytes()?))
            .expect("owned spatial allocation bytes fit address space");
        let mutation = if tree.is_empty() {
            self.spatial.remove_with_work(&key).1
        } else {
            self.spatial.insert_with_work(key, tree)
        };
        record_map(work, mutation);
    }
}

fn spatial_key(row: UiMountedHitTestMechanic) -> (UiSurfaceBindingGeneration, u8) {
    (row.binding(), row.bounds().coordinate_space() as u8)
}

fn region(row: UiMountedHitTestMechanic) -> [f64; 4] {
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let x = f64::from(bounds.x()).max(f64::from(clip.x()));
    let y = f64::from(bounds.y()).max(f64::from(clip.y()));
    let right = (f64::from(bounds.x()) + f64::from(bounds.width()))
        .min(f64::from(clip.x()) + f64::from(clip.width()))
        .max(x);
    let bottom = (f64::from(bounds.y()) + f64::from(bounds.height()))
        .min(f64::from(clip.y()) + f64::from(clip.height()))
        .max(y);
    [x, y, right, bottom]
}

fn record_map(work: &mut UiHitTestSpatialWork, mutation: UiPersistentIndexMutationWork) {
    work.map_key_probes += mutation.key_probes();
    work.map_node_copies += mutation.node_copies();
}

fn row_digest(row: UiMountedHitTestMechanic) -> u64 {
    row.semantic_digest()
        .wrapping_mul(0x9e37_79b1_85eb_ca87)
        .rotate_left(19)
}

#[cfg(test)]
mod tests;
