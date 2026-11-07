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
    entries: Selections,
    empty: im::OrdSet<DerivedIndexId>,
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
                entries: Selections::new(),
                empty: im::OrdSet::new(),
            });
        basis.empty.remove(&generation.index_id);
        basis.entries.insert(generation.index_id, generation);
        self.install(basis);
    }
    /// Cost: explicit derived-state discard visits its retained basis selections.
    pub(super) fn discard(&mut self, index: DerivedIndexId) {
        let changed: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, basis)| basis.entries.contains_key(&index) || basis.empty.contains(&index))
            .map(|(key, basis)| (key.clone(), basis.as_ref().clone()))
            .collect();
        for (key, mut basis) in changed {
            basis.entries.remove(&index);
            basis.empty.remove(&index);
            if basis.entries.is_empty() && basis.empty.is_empty() {
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
    pub(crate) fn generation_is_empty(&self, index: DerivedIndexId) -> bool {
        self.empty.contains(&index)
    }

    pub(crate) fn generation(&self, index: DerivedIndexId) -> Option<Arc<DerivedIndexGeneration>> {
        self.entries.get(&index).map(Arc::clone)
    }
    #[cfg(test)]
    pub(super) fn contains_generation(
        &self,
        index: DerivedIndexId,
        generation: crate::indexes::data::DerivedIndexGenerationId,
    ) -> bool {
        self.entries
            .get(&index)
            .is_some_and(|entry| entry.generation_id == generation)
    }
    pub(crate) fn remove_admitted(
        &mut self,
        index: DerivedIndexId,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let (units, bytes) = payment::inner(self.entries.len().saturating_add(self.empty.len()))
            .ok_or(Denial::WorkBudgetExceeded)?;
        prepare(units, bytes)?;
        if self.entries.contains_key(&index) {
            self.entries.remove(&index);
        } else {
            self.empty.remove(&index);
        }
        Ok(())
    }
    pub(crate) fn replace_admitted(
        &mut self,
        generation: Arc<DerivedIndexGeneration>,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let (units, bytes) = payment::inner(
            self.entries
                .len()
                .saturating_add(self.empty.len())
                .saturating_add(1),
        )
        .ok_or(Denial::WorkBudgetExceeded)?;
        prepare(units, bytes)?;
        self.replace(generation);
        Ok(())
    }
    pub(crate) fn replace(&mut self, generation: Arc<DerivedIndexGeneration>) {
        if self.empty.contains(&generation.index_id) {
            self.empty.remove(&generation.index_id);
        }
        self.entries.insert(generation.index_id, generation);
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
            entries: previous.entries.clone(),
            empty: previous.empty.clone(),
        })
    }
    pub(crate) fn bind_published_generations(
        &self,
        basis: &mut PreparedIndexBasisSelection,
        generations: &[DerivedIndexGeneration],
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let state = self.state.read();
        for generation in generations {
            if let Some(payload) = state.generations.generation(generation.generation_id) {
                basis.replace_admitted(payload, prepare)?;
            }
        }
        Ok(())
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
