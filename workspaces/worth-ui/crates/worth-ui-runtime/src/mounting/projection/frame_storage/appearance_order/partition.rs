use super::{record_map, Row, UiMountedAppearanceOrderDenial as Denial, Update};
use crate::mounting::spatial_index::{
    UiMountedSpatialBudget, UiMountedSpatialQueryDenial, UiMountedSpatialTree, UiMountedSpatialWork,
};
use crate::runtime::persistent_index::UiPersistentOrdMap;
use worth_ui_host_contract::UiMountedInstanceIdentity;

#[derive(Clone, Default)]
pub(super) struct Surface {
    rows: UiPersistentOrdMap<UiMountedInstanceIdentity, Box<[Row]>>,
    partitions: UiPersistentOrdMap<super::row::PartitionKey, [UiMountedSpatialTree; 2]>,
    tree_bytes: usize,
    row_payload_bytes: usize,
}

impl Surface {
    pub(super) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(super) fn matches(&self, update: &Update, work: &mut UiMountedSpatialWork) -> bool {
        let (rows, probes) = self.rows.get_with_probes(&update.instance);
        work.map_key_probes += probes;
        rows.map_or(update.rows.is_empty(), |rows| {
            rows.as_ref() == update.rows.as_slice()
        })
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.rows
            .retained_structural_bytes()
            .expect("bounded appearance membership")
            + self
                .partitions
                .retained_structural_bytes()
                .expect("bounded appearance partitions")
            + self.tree_bytes
            + self.row_payload_bytes
    }

    pub(super) fn remove(
        &mut self,
        instance: UiMountedInstanceIdentity,
        work: &mut UiMountedSpatialWork,
    ) {
        let (rows, probes) = self.rows.get_with_probes(&instance);
        work.map_key_probes += probes;
        let Some(rows) = rows.cloned() else {
            return;
        };
        self.row_payload_bytes -= rows.len() * std::mem::size_of::<Row>();
        for row in rows {
            let (partition, probes) = self.partitions.get_with_probes(&row.key);
            work.map_key_probes += probes;
            let mut partition = partition
                .expect("retained row has its spatial partition")
                .clone();
            self.tree_bytes -= tree_bytes(&partition);
            work.merge(
                partition[row.family]
                    .replace(instance, Some(row.bounds), None)
                    .expect("completed integer bounds remain finite"),
            );
            self.tree_bytes += tree_bytes(&partition);
            let mutation = if partition.iter().all(UiMountedSpatialTree::is_empty) {
                self.partitions.remove_with_work(&row.key).1
            } else {
                self.partitions.insert_with_work(row.key, partition)
            };
            record_map(work, mutation);
        }
        record_map(work, self.rows.remove_with_work(&instance).1);
    }

    pub(super) fn insert(
        &mut self,
        update: &Update,
        work: &mut UiMountedSpatialWork,
    ) -> Result<(), Denial> {
        for row in &update.rows {
            let (partition, probes) = self.partitions.get_with_probes(&row.key);
            work.map_key_probes += probes;
            let mut partition = partition.cloned().unwrap_or_default();
            let previous_bytes = tree_bytes(&partition);
            for tree in &partition {
                let result = tree.intersecting(
                    row.bounds,
                    UiMountedSpatialBudget {
                        node_visits: 1_024,
                        candidates: 256,
                    },
                );
                let query = match result {
                    Ok(query) => query,
                    Err(UiMountedSpatialQueryDenial::InvalidGeometry) => {
                        return Err(Denial::InvalidGeometry)
                    }
                    Err(
                        UiMountedSpatialQueryDenial::NodeBudget { work: measured }
                        | UiMountedSpatialQueryDenial::CandidateBudget { work: measured },
                    ) => {
                        work.merge(measured);
                        return Err(Denial::QueryBudgetExceeded);
                    }
                };
                work.merge(query.work);
                if let Some(other) = query
                    .instances
                    .into_iter()
                    .find(|instance| *instance != update.instance)
                {
                    return Err(Denial::Ambiguous {
                        surface: update.surface,
                        first: other,
                        second: update.instance,
                        rank: row.key.rank,
                    });
                }
            }
            work.merge(
                partition[row.family]
                    .replace(update.instance, None, Some(row.bounds))
                    .map_err(|_| Denial::InvalidGeometry)?,
            );
            self.tree_bytes = self.tree_bytes - previous_bytes + tree_bytes(&partition);
            record_map(work, self.partitions.insert_with_work(row.key, partition));
        }
        if !update.rows.is_empty() {
            self.row_payload_bytes += update.rows.len() * std::mem::size_of::<Row>();
            record_map(
                work,
                self.rows
                    .insert_with_work(update.instance, update.rows.clone().into_boxed_slice()),
            );
        }
        Ok(())
    }
}

fn tree_bytes(trees: &[UiMountedSpatialTree; 2]) -> usize {
    trees
        .iter()
        .map(|tree| {
            tree.retained_structural_bytes()
                .expect("bounded appearance spatial membership")
        })
        .sum()
}
