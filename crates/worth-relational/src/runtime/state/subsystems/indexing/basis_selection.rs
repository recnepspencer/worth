//! Exact-root applicability carries persistent selections.
mod admission;
mod checkpoint;
mod empty_basis;
mod payment;
use super::{IndexingState, IndexingSubsystem};
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexId, DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::schema::data::SchemaVersionId;
use std::sync::Arc;

type Key = (Option<Arc<BranchId>>, VersionId);
type Selections = im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>;

#[derive(Clone, Debug)]
pub(crate) struct PreparedIndexBasisSelection {
    pub(crate) branch: BranchId,
    pub(crate) commit: CommitId,
    pub(crate) version: VersionId,
    pub(crate) schema: SchemaVersionId,
    state: Arc<super::IndexSelectionState>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct BasisSelections {
    entries: std::collections::BTreeMap<Key, Arc<PreparedIndexBasisSelection>>,
    retirement: im::Vector<Key>,
    versions: std::collections::BTreeMap<VersionId, VersionSelections>,
}
#[derive(Clone, Debug, Default)]
struct VersionSelections {
    global: Option<Arc<PreparedIndexBasisSelection>>,
    branches: std::collections::BTreeMap<Arc<BranchId>, Arc<PreparedIndexBasisSelection>>,
}
impl BasisSelections {
    pub(super) fn get(
        &self,
        branch: Option<&BranchId>,
        version: VersionId,
    ) -> Option<&Arc<PreparedIndexBasisSelection>> {
        let version = self.versions.get(&version)?;
        match branch {
            Some(branch) => version.branches.get(branch),
            None => version.global.as_ref(),
        }
    }
    pub(super) fn install(&mut self, basis: PreparedIndexBasisSelection) {
        let version = basis.version;
        let branch = basis.branch.clone();
        let basis = Arc::new(basis);
        self.replace_lookup((None, version), Some(Arc::clone(&basis)));
        self.replace_lookup((Some(Arc::new(branch)), version), Some(basis));
    }
    pub(super) fn insert_generation(&mut self, generation: Arc<DerivedIndexGeneration>) {
        let mut basis = self
            .get(
                Some(&generation.source_branch_id),
                generation.applicability.version_id,
            )
            .map(|basis| basis.as_ref().clone())
            .unwrap_or_else(|| PreparedIndexBasisSelection {
                branch: generation.source_branch_id.clone(),
                commit: generation.source_commit_id,
                version: generation.applicability.version_id,
                schema: generation.applicability.schema_version,
                state: Arc::new(super::IndexSelectionState::default()),
            });
        basis.replace(generation);
        self.install(basis);
    }
    /// Cost: explicit derived-state discard visits its retained basis selections.
    pub(super) fn discard(&mut self, index: DerivedIndexId) {
        let changed: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, basis)| {
                basis.state.entries.contains_key(&index) || basis.state.empty.contains(&index)
            })
            .map(|(key, basis)| (key.clone(), basis.as_ref().clone()))
            .collect();
        for (key, mut basis) in changed {
            let state = Arc::make_mut(&mut basis.state);
            state.entries.remove(&index);
            state.empty.remove(&index);
            if basis.state.entries.is_empty() && basis.state.empty.is_empty() {
                self.replace_lookup(key, None);
            } else {
                self.replace_lookup(key, Some(Arc::new(basis)));
            }
        }
    }
    /// Cost: a fixed retirement quota; the history owner decides actual custody.
    pub(super) fn retire(
        &mut self,
        maximum: usize,
        mut held: impl FnMut(Option<&BranchId>, VersionId, SchemaVersionId) -> bool,
    ) {
        let visits = maximum.min(self.retirement.len());
        for _ in 0..visits {
            let key = self
                .retirement
                .pop_front()
                .expect("bounded queued retirement visit");
            let Some(basis) = self.entries.get(&key) else {
                continue;
            };
            if held(Some(&basis.branch), basis.version, basis.schema) {
                // FIFO custody checks are fair even while forks at an old version arrive.
                self.retirement.push_back(key);
            } else {
                self.replace_lookup(key, None);
            }
        }
    }
    /// cost-lane: explicit reclamation scans the retained exact-basis catalog.
    pub(super) fn reclaim(&mut self, retained: &crate::history::retention::RetainedIndexRoots) {
        let removed: Vec<_> = self
            .entries
            .iter()
            .filter(|(key, basis)| !match &key.0 {
                None => {
                    retained
                        .historical_versions
                        .contains(&(basis.version, basis.schema))
                        || retained.live.contains(&(
                            basis.branch.clone(),
                            basis.version,
                            basis.schema,
                        ))
                        || retained.retired.contains(&(
                            basis.branch.clone(),
                            basis.version,
                            basis.schema,
                        ))
                }
                Some(branch) => {
                    retained
                        .historical_versions
                        .contains(&(basis.version, basis.schema))
                        || retained.live.contains(&(
                            branch.as_ref().clone(),
                            basis.version,
                            basis.schema,
                        ))
                        || retained.retired.contains(&(
                            branch.as_ref().clone(),
                            basis.version,
                            basis.schema,
                        ))
                }
            })
            .map(|(key, ..)| key.clone())
            .collect();
        for key in removed {
            self.replace_lookup(key, None);
        }
        self.retirement = self.entries.keys().cloned().collect();
    }
    pub(super) fn merge(&mut self, previous: &Self) {
        for (key, basis) in &previous.entries {
            if !self.entries.contains_key(key) {
                self.replace_lookup(key.clone(), Some(Arc::clone(basis)));
            }
        }
    }
    fn replace_lookup(&mut self, key: Key, basis: Option<Arc<PreparedIndexBasisSelection>>) {
        if let Some(basis) = basis {
            if self.entries.insert(key.clone(), basis).is_none() {
                self.retirement.push_back(key.clone());
            }
        } else {
            self.entries.remove(&key);
        }
        let version = self.versions.entry(key.1).or_default();
        match &key.0 {
            None => version.global = self.entries.get(&key).map(Arc::clone),
            Some(branch) => {
                if let Some(basis) = self.entries.get(&key) {
                    version
                        .branches
                        .insert(Arc::clone(branch), Arc::clone(basis));
                } else {
                    version.branches.remove(branch.as_ref());
                }
            }
        }
        if version.global.is_none() && version.branches.is_empty() {
            self.versions.remove(&key.1);
        }
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
impl PreparedIndexBasisSelection {
    pub(crate) fn selection_state(&self) -> Arc<super::IndexSelectionState> {
        Arc::clone(&self.state)
    }
    pub(crate) fn retain_state(&mut self, state: Arc<super::IndexSelectionState>) {
        self.state = state;
    }

    pub(crate) fn generation_is_empty(&self, index: DerivedIndexId) -> bool {
        self.state.empty.contains(&index)
    }

    pub(crate) fn generation(&self, index: DerivedIndexId) -> Option<Arc<DerivedIndexGeneration>> {
        self.state.entries.get(&index).map(Arc::clone)
    }
    #[cfg(test)]
    pub(super) fn contains_generation(
        &self,
        index: DerivedIndexId,
        generation: crate::indexes::data::DerivedIndexGenerationId,
    ) -> bool {
        self.state
            .entries
            .get(&index)
            .is_some_and(|entry| entry.generation_id == generation)
    }
    pub(crate) fn remove_admitted(
        &mut self,
        index: DerivedIndexId,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let (units, bytes) = payment::inner(
            self.state
                .entries
                .len()
                .saturating_add(self.state.empty.len()),
        )
        .ok_or(Denial::WorkBudgetExceeded)?;
        let state_bytes = if Arc::strong_count(&self.state) > 1 {
            std::mem::size_of::<super::IndexSelectionState>() + 2 * std::mem::size_of::<usize>()
        } else {
            0
        };
        prepare(units + 1, bytes + state_bytes as u64)?;
        let state = Arc::make_mut(&mut self.state);
        if state.entries.contains_key(&index) {
            state.entries.remove(&index);
        } else {
            state.empty.remove(&index);
        }
        Ok(())
    }
    pub(crate) fn replace_admitted(
        &mut self,
        generation: Arc<DerivedIndexGeneration>,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let (units, bytes) = payment::inner(
            self.state
                .entries
                .len()
                .saturating_add(self.state.empty.len())
                .saturating_add(1),
        )
        .ok_or(Denial::WorkBudgetExceeded)?;
        let state_bytes = if Arc::strong_count(&self.state) > 1 {
            std::mem::size_of::<super::IndexSelectionState>() + 2 * std::mem::size_of::<usize>()
        } else {
            0
        };
        prepare(units + 1, bytes + state_bytes as u64)?;
        self.replace(generation);
        Ok(())
    }
    pub(crate) fn replace(&mut self, generation: Arc<DerivedIndexGeneration>) {
        let state = Arc::make_mut(&mut self.state);
        if state.empty.contains(&generation.index_id) {
            state.empty.remove(&generation.index_id);
        }
        state.entries.insert(generation.index_id, generation);
    }
}
impl IndexingState {
    pub(super) fn record_basis_generation(
        &mut self,
        generation: crate::indexes::data::DerivedIndexGenerationId,
    ) {
        let payload = self
            .generations
            .generation(generation)
            .expect("publication just installed its generation");
        self.bases.insert_generation(payload);
    }
}
impl IndexingSubsystem {
    /// Cost: two persistent map descents. The predecessor's actual root axes are supplied by the owner.
    pub(crate) fn prepare_basis_selection(
        &self,
        branch: &BranchId,
        commit: CommitId,
        version: VersionId,
        schema: SchemaVersionId,
        previous_branch: &BranchId,
        previous_commit: CommitId,
        previous_version: VersionId,
        deferred: bool,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<PreparedIndexBasisSelection, Denial> {
        let state = self.state.read();
        let width = previous_branch.0.len().max(branch.0.len()) as u64;
        let levels = u64::from(usize::BITS - state.bases.len().leading_zeros()) + 1;
        prepare(
            levels
                .saturating_mul(128)
                .saturating_mul(width.saturating_add(1)),
            0,
        )?;
        let maximum = if deferred {
            usize::MAX
        } else {
            state.bases.len().saturating_add(2)
        };
        let (units, bytes) = payment::outer(maximum, width).ok_or(Denial::WorkBudgetExceeded)?;
        prepare(units, bytes)?;
        let previous = state
            .bases
            .get(Some(previous_branch), previous_version)
            .ok_or(Denial::ColdReconstructionRequired)?;
        if previous.commit != previous_commit || previous.schema != schema {
            return Err(Denial::ColdReconstructionRequired);
        }
        Ok(PreparedIndexBasisSelection {
            branch: branch.clone(),
            commit,
            version,
            schema,
            state: Arc::clone(&previous.state),
        })
    }
    pub(crate) fn install_basis_selection(&self, basis: PreparedIndexBasisSelection) {
        self.state.write().bases.install(basis);
    }
}

impl IndexingSubsystem {
    pub(crate) fn publish_generation(&self, generation: DerivedIndexGeneration) {
        let mut state = self.state.write();
        let id = generation.generation_id;
        state.generations.publish(generation);
        state.record_basis_generation(id);
    }
}

impl IndexingSubsystem {
    #[cfg(test)]
    pub(crate) fn generation_applies(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
        generation: crate::indexes::data::DerivedIndexGenerationId,
    ) -> bool {
        let state = self.state.read();
        state.bases.get(branch, version).is_some_and(|basis| {
            basis.schema == schema && basis.contains_generation(index, generation)
        })
    }
}
