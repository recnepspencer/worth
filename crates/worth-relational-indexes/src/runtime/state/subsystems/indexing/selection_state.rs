//! Persistent index bindings carried by an exact root and its global projection.
use std::sync::Arc;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId;

#[derive(Clone, Debug, Default)]
pub(crate) struct IndexSelectionState {
    pub(super) exact: BasisBindings,
    pub(super) global: BasisBindings,
}

#[derive(Clone, Debug, Default)]
pub(super) struct BasisBindings {
    pub(super) generations: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) empty: im::OrdSet<DerivedIndexId>,
}

impl BasisBindings {
    pub(super) fn len(&self) -> usize {
        self.generations.len().saturating_add(self.empty.len())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.generations.is_empty() && self.empty.is_empty()
    }

    pub(super) fn contains(&self, index: &DerivedIndexId) -> bool {
        self.generations.contains_key(index) || self.empty.contains(index)
    }

    fn remove(&mut self, index: &DerivedIndexId) {
        self.generations.remove(index);
        self.empty.remove(index);
    }

    fn replace(&mut self, generation: Arc<DerivedIndexGeneration>) {
        self.empty.remove(&generation.index_id);
        self.generations.insert(generation.index_id, generation);
    }
}

impl IndexSelectionState {
    pub(super) fn remove(&mut self, index: &DerivedIndexId) {
        self.exact.remove(index);
        self.global.remove(index);
    }

    pub(super) fn replace(&mut self, generation: Arc<DerivedIndexGeneration>, global: bool) {
        if global {
            self.global.replace(Arc::clone(&generation));
        }
        self.exact.replace(generation);
    }

    pub(super) fn fill_missing(&mut self, restored: &Self) {
        // A live exact binding also defeats the restored global projection.
        for (index, generation) in &restored.exact.generations {
            if !self.exact.contains(index) {
                self.replace(
                    Arc::clone(generation),
                    restored.global.generations.contains_key(index),
                );
            }
        }
        for index in &restored.exact.empty {
            if !self.exact.contains(index) {
                self.exact.empty.insert(*index);
                if restored.global.empty.contains(index) {
                    self.global.empty.insert(*index);
                }
            }
        }
    }
}
