use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPaintCommandChange, UiMountedSemanticTextMechanic,
    UiSemanticTextSlot,
};

use super::super::semantic_text::{
    UiMountedCollectionTextKey, UiMountedQualifiedSemanticText,
    UiMountedSemanticTextCompletionContext, UiMountedSemanticTextSeed,
    UiMountedSemanticTextSeedContent, UiMountedSemanticTextSeedTransition,
};
use super::UiMountedProjectionNodeRecord;
use crate::mounting::{UiMountedCollectionTextChange, UiMountedProjectionDenial};
use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentRankedSequence};

mod capacity;
mod diff;
mod key;
mod layout_index;
mod layout_reconstruction;
mod paint_only;
mod sparse_update;
#[cfg(test)]
mod test_views;

use diff::diff_rows;
use key::row_digest;
use layout_index::UiMountedQualifiedLayoutIndex;
use sparse_update::{apply_posture_update, apply_row_update};

#[derive(Clone, Default)]
pub(in crate::mounting) struct UiMountedSemanticMechanicSource {
    by_instance: UiPersistentOrdMap<UiMountedInstanceIdentity, UiMountedSemanticMechanicRows>,
    by_layout: UiMountedQualifiedLayoutIndex,
    row_count: usize,
    byte_count: usize,
    digest: u64,
}

#[derive(Clone, Default)]
struct UiMountedSemanticMechanicRows {
    rows: UiPersistentOrdMap<UiMountedSemanticMechanicKey, UiMountedQualifiedSemanticText>,
    order: UiPersistentRankedSequence<UiMountedSemanticMechanicKey>,
    digest: u64,
    byte_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UiMountedSemanticMechanicKey {
    slot: UiSemanticTextSlot,
    collection: Option<[u8; 32]>,
}

pub(super) struct UiMountedSemanticMechanicUpdate {
    pub(super) rows_materialized: usize,
    pub(super) command_changes: Vec<UiMountedPaintCommandChange>,
}

#[derive(Default)]
struct UiMountedSparseSemanticChanges {
    commands: Vec<UiMountedPaintCommandChange>,
    layouts: Vec<(
        UiMountedQualifiedSemanticText,
        UiMountedQualifiedSemanticText,
    )>,
}

impl UiMountedSemanticMechanicSource {
    pub(super) fn replace_instance(
        &mut self,
        instance: UiMountedInstanceIdentity,
        rows: Vec<UiMountedQualifiedSemanticText>,
    ) -> Result<UiMountedSemanticMechanicUpdate, UiMountedProjectionDenial> {
        let predecessor = self.by_instance.get(&instance).cloned().unwrap_or_default();
        let successor = UiMountedSemanticMechanicRows::from_rows(rows)?;
        let command_changes = diff_rows(&predecessor, &successor);
        self.row_count = self
            .row_count
            .checked_sub(predecessor.len())
            .and_then(|count| count.checked_add(successor.len()))
            .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
        self.replace_capacity(&predecessor, &successor)?;
        self.digest ^= predecessor.digest ^ successor.digest;
        self.by_layout.remove_rows(&predecessor);
        self.by_layout.insert_rows(&successor);
        let rows_materialized = successor.len();
        if successor.len() == 0 {
            self.by_instance.remove(&instance);
        } else {
            self.by_instance.insert(instance, successor);
        }
        Ok(UiMountedSemanticMechanicUpdate {
            rows_materialized,
            command_changes,
        })
    }

    pub(super) fn remove_instance(
        &mut self,
        instance: UiMountedInstanceIdentity,
    ) -> Vec<UiMountedPaintCommandChange> {
        let Some(predecessor) = self.by_instance.get(&instance).cloned() else {
            return Vec::new();
        };
        self.row_count = self.row_count.saturating_sub(predecessor.len());
        self.remove_capacity(&predecessor);
        self.digest ^= predecessor.digest;
        self.by_layout.remove_rows(&predecessor);
        self.by_instance.remove(&instance);
        predecessor
            .iter()
            .map(|row| {
                UiMountedPaintCommandChange::Remove(
                    worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(row),
                )
            })
            .collect()
    }

    pub(super) fn apply_collection_patch(
        &mut self,
        context: &UiMountedSemanticTextCompletionContext<'_>,
        node: &UiMountedProjectionNodeRecord,
        seed: &UiMountedSemanticTextSeed,
    ) -> Option<Result<UiMountedSemanticMechanicUpdate, UiMountedProjectionDenial>> {
        let UiMountedSemanticTextSeedTransition::CollectionPatch(changes) = seed.transition()
        else {
            return None;
        };
        if !changes
            .iter()
            .all(|change| matches!(change, UiMountedCollectionTextChange::Update(_)))
        {
            return None;
        }
        Some(self.apply_collection_updates(context, node, seed, changes))
    }

    fn apply_collection_updates(
        &mut self,
        context: &UiMountedSemanticTextCompletionContext<'_>,
        node: &UiMountedProjectionNodeRecord,
        seed: &UiMountedSemanticTextSeed,
        changes: &[UiMountedCollectionTextChange],
    ) -> Result<UiMountedSemanticMechanicUpdate, UiMountedProjectionDenial> {
        let instance = node.receipt.mounted_instance();
        let mut rows = self
            .by_instance
            .get(&instance)
            .cloned()
            .ok_or(UiMountedProjectionDenial::MissingSemanticCollectionPredecessor)?;
        let UiMountedSemanticTextSeedContent::Collection(source) = seed.content() else {
            return Err(UiMountedProjectionDenial::SemanticTextShapeMismatch);
        };
        let mut sparse_changes = UiMountedSparseSemanticChanges::default();
        for change in changes {
            let UiMountedCollectionTextChange::Update(row) = change else {
                unreachable!("sparse collection path admits updates only")
            };
            apply_row_update(
                context,
                node,
                seed,
                source,
                row,
                &mut rows,
                &mut sparse_changes,
            )?;
        }
        apply_posture_update(context, node, seed, &mut rows, &mut sparse_changes)?;
        let predecessor_digest = self
            .by_instance
            .get(&instance)
            .expect("sparse update has predecessor rows")
            .digest;
        self.update_capacity(instance, &rows)?;
        self.digest ^= predecessor_digest ^ rows.digest;
        for (predecessor, successor) in &sparse_changes.layouts {
            self.by_layout.replace_row(predecessor, successor);
        }
        self.by_instance.insert(instance, rows);
        Ok(UiMountedSemanticMechanicUpdate {
            rows_materialized: sparse_changes.commands.len(),
            command_changes: sparse_changes.commands,
        })
    }

    pub(in crate::mounting) const fn len(&self) -> usize {
        self.row_count
    }

    pub(super) const fn digest(&self) -> u64 {
        self.digest
    }

    pub(super) fn rows_for_instance(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> impl Iterator<Item = &UiMountedSemanticTextMechanic> {
        self.by_instance
            .get(&instance)
            .into_iter()
            .flat_map(|rows| rows.iter().map(UiMountedQualifiedSemanticText::mechanic))
    }

    pub(super) fn retained_rows_for_instance(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> impl Iterator<Item = &UiMountedQualifiedSemanticText> {
        self.by_instance
            .get(&instance)
            .into_iter()
            .flat_map(UiMountedSemanticMechanicRows::iter)
    }

    pub(super) fn retained_iter(&self) -> impl Iterator<Item = &UiMountedQualifiedSemanticText> {
        self.by_instance.iter().flat_map(|(_, rows)| rows.iter())
    }

    pub(in crate::mounting) fn visual_mechanics(
        &self,
    ) -> impl Iterator<Item = &UiMountedSemanticTextMechanic> {
        self.retained_iter()
            .map(UiMountedQualifiedSemanticText::mechanic)
    }

    pub(in crate::mounting) fn retained_structural_bytes(&self) -> Option<usize> {
        let row_structures = self
            .by_instance
            .iter()
            .try_fold(0usize, |bytes, (_, rows)| {
                bytes.checked_add(rows.retained_structural_bytes()?)
            })?;
        std::mem::size_of::<Self>()
            .checked_add(self.by_instance.retained_structural_bytes()?)?
            .checked_add(row_structures)?
            .checked_add(self.by_layout.retained_structural_bytes()?)
    }

    pub(super) fn qualified_layout(
        &self,
        identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>> {
        self.by_layout.get(identity)
    }

    fn rebuild_layout_index(&mut self) {
        self.by_layout = UiMountedQualifiedLayoutIndex::default();
        for (_, rows) in self.by_instance.iter() {
            self.by_layout.insert_rows(rows);
        }
    }

    #[cfg(test)]
    pub(super) fn qualified_layout_for(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: worth_ui_host_contract::UiSemanticTextSlot,
    ) -> Option<&std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>> {
        self.by_instance
            .get(&instance)?
            .iter()
            .find(|row| row.slot() == slot)
            .and_then(UiMountedQualifiedSemanticText::qualified_layout)
    }

    pub(super) fn replace_all(
        &mut self,
        rows: impl IntoIterator<Item = UiMountedQualifiedSemanticText>,
    ) -> Result<(), UiMountedProjectionDenial> {
        *self = Self::default();
        let mut grouped = std::collections::BTreeMap::<_, Vec<_>>::new();
        for row in rows {
            grouped.entry(row.mounted_instance()).or_default().push(row);
        }
        for (instance, rows) in grouped {
            self.replace_instance(instance, rows)?;
        }
        Ok(())
    }
}

impl UiMountedSemanticMechanicRows {
    fn from_rows(
        rows: Vec<UiMountedQualifiedSemanticText>,
    ) -> Result<Self, UiMountedProjectionDenial> {
        let mut source = Self::default();
        for row in rows {
            let key = UiMountedSemanticMechanicKey::for_row(&row);
            if source.rows.get(&key).is_some() {
                return Err(UiMountedProjectionDenial::DrawableSourceCoverageMismatch);
            }
            source
                .order
                .insert(source.order.len(), key)
                .map_err(|()| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            source.rows.insert(key, row);
            source.byte_count = source
                .byte_count
                .checked_add(
                    source
                        .rows
                        .get(&key)
                        .expect("inserted semantic mechanic")
                        .text()
                        .len(),
                )
                .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            source.digest ^= row_digest(
                source
                    .rows
                    .get(&key)
                    .expect("inserted semantic mechanic")
                    .semantic_digest(),
            );
        }
        Ok(source)
    }

    fn len(&self) -> usize {
        self.order.len()
    }

    fn iter(&self) -> impl ExactSizeIterator<Item = &UiMountedQualifiedSemanticText> {
        self.order.iter().map(|key| {
            self.rows
                .get(key)
                .expect("semantic mechanic order names an indexed row")
        })
    }

    fn retained_structural_bytes(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.rows.retained_structural_bytes()?)?
            .checked_add(self.order.retained_structural_bytes()?)
    }

    fn replace(
        &mut self,
        key: UiMountedSemanticMechanicKey,
        row: UiMountedQualifiedSemanticText,
    ) -> Result<Option<UiMountedQualifiedSemanticText>, UiMountedProjectionDenial> {
        let predecessor = self
            .rows
            .get(&key)
            .cloned()
            .ok_or(UiMountedProjectionDenial::InvalidSemanticCollectionPatch)?;
        self.byte_count = self
            .byte_count
            .checked_sub(predecessor.text().len())
            .and_then(|count| count.checked_add(row.text().len()))
            .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
        self.digest ^=
            row_digest(predecessor.semantic_digest()) ^ row_digest(row.semantic_digest());
        self.rows.insert(key, row);
        Ok(Some(predecessor))
    }
}
