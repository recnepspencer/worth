use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::history::data::BranchId;

use super::{RelationalBranchCellCheckpoint, RelationalBranchReferenceCell};

#[derive(Debug, Default)]
struct RelationalBranchRegistryState {
    cells: HashMap<BranchId, RelationalBranchReferenceCell>,
    retired_names: HashSet<BranchId>,
    reserved_targets: HashSet<BranchId>,
}

/// Cloneable owner of runtime-affine branch-reference cells and target names.
#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalBranchReferenceRegistry {
    state: Arc<RwLock<RelationalBranchRegistryState>>,
}

/// Move-only owner-issued custody for one exact Relational fork destination.
///
/// The registry and active bit are intentionally private. A caller can carry
/// or drop this reservation, but cannot forge, clone, serialize, or install it
/// outside the owner that issued it.
#[derive(Debug)]
pub struct RelationalForkTargetReservation {
    registry: RelationalBranchReferenceRegistry,
    branch_id: BranchId,
    active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalForkTargetReservationDenial {
    Duplicate,
    Retired,
}

impl RelationalBranchReferenceRegistry {
    pub(crate) fn owns_reservation(&self, reservation: &RelationalForkTargetReservation) -> bool {
        self.same_owner(&reservation.registry)
    }

    fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let state = self.read();
        Self {
            state: Arc::new(RwLock::new(RelationalBranchRegistryState {
                cells: state
                    .cells
                    .iter()
                    .map(|(branch_id, cell)| (branch_id.clone(), cell.detached_owner_snapshot()))
                    .collect(),
                retired_names: state.retired_names.clone(),
                reserved_targets: HashSet::new(),
            })),
        }
    }

    pub(crate) fn from_main(main: RelationalBranchReferenceCell) -> Self {
        let branch_id = main.identity().branch_id().clone();
        Self {
            state: Arc::new(RwLock::new(RelationalBranchRegistryState {
                cells: HashMap::from([(branch_id, main)]),
                ..Default::default()
            })),
        }
    }

    pub(crate) fn get(&self, branch_id: &BranchId) -> Option<RelationalBranchReferenceCell> {
        self.read().cells.get(branch_id).cloned()
    }

    pub(crate) fn insert(&self, cell: RelationalBranchReferenceCell) {
        self.write()
            .cells
            .insert(cell.identity().branch_id().clone(), cell);
    }

    pub(crate) fn remove(&self, branch_id: &BranchId) -> Option<RelationalBranchReferenceCell> {
        self.write().cells.remove(branch_id)
    }

    pub(crate) fn contains(&self, branch_id: &BranchId) -> bool {
        self.read().cells.contains_key(branch_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.read().cells.len()
    }

    pub(crate) fn keys(&self) -> Vec<BranchId> {
        self.read().cells.keys().cloned().collect()
    }

    pub(crate) fn values(&self) -> Vec<RelationalBranchReferenceCell> {
        self.read().cells.values().cloned().collect()
    }

    pub(crate) fn take_all(&self) -> BTreeMap<BranchId, RelationalBranchReferenceCell> {
        std::mem::take(&mut self.write().cells)
            .into_iter()
            .collect()
    }

    pub(crate) fn restore_all(&self, cells: BTreeMap<BranchId, RelationalBranchReferenceCell>) {
        self.write().cells = cells.into_iter().collect();
    }

    pub(crate) fn checkpoints(&self) -> Vec<RelationalBranchCellCheckpoint> {
        let mut checkpoints = self
            .read()
            .cells
            .values()
            .map(RelationalBranchReferenceCell::checkpoint)
            .collect::<Vec<_>>();
        checkpoints.sort_by(|left, right| left.branch_id.cmp(&right.branch_id));
        checkpoints
    }

    /// Every branch's lifecycle posture, in branch order.
    pub(crate) fn lifecycle_postures(
        &self,
    ) -> Vec<crate::durability::data::DurableBranchLifecycleImage> {
        let mut postures = self
            .read()
            .cells
            .iter()
            .map(
                |(branch_id, cell)| crate::durability::data::DurableBranchLifecycleImage {
                    branch_id: branch_id.clone(),
                    posture: cell.lifecycle_posture(),
                },
            )
            .collect::<Vec<_>>();
        postures.sort_by(|left, right| left.branch_id.cmp(&right.branch_id));
        postures
    }

    /// Whether `branch_id` is retired already or fits beside the others.
    pub(crate) fn admits_name_retirement(
        &self,
        branch_id: &BranchId,
        maximum_names: usize,
    ) -> bool {
        let state = self.read();
        state.retired_names.contains(branch_id) || state.retired_names.len() < maximum_names
    }

    pub(crate) fn retire_name(&self, branch_id: BranchId) {
        self.write().retired_names.insert(branch_id);
    }

    /// Every retired name, sorted.
    pub(crate) fn retired_names(&self) -> Vec<BranchId> {
        let mut names = self
            .read()
            .retired_names
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Replace the retired names with `names`. A name a registered branch
    /// still uses is refused: a branch is retired only by its deletion, which
    /// removes it or marks it deleting.
    pub(crate) fn restore_retired_names(&self, names: &[BranchId]) -> Result<(), String> {
        let mut state = self.write();
        let mut restored = HashSet::with_capacity(names.len());
        for name in names {
            let deleting = state.cells.get(name).is_some_and(|cell| {
                cell.lifecycle_posture() == super::RelationalBranchLifecyclePosture::Deleting
            });
            if state.cells.contains_key(name) && !deleting {
                return Err(format!(
                    "retired branch name `{}` names a live branch",
                    name.0
                ));
            }
            if !restored.insert(name.clone()) {
                return Err(format!(
                    "retired branch name `{}` is recorded twice",
                    name.0
                ));
            }
        }
        state.retired_names = restored;
        Ok(())
    }

    pub(crate) fn reserve_fork_target(
        &self,
        branch_id: BranchId,
    ) -> Result<RelationalForkTargetReservation, RelationalForkTargetReservationDenial> {
        let mut state = self.write();
        if state.retired_names.contains(&branch_id) {
            return Err(RelationalForkTargetReservationDenial::Retired);
        }
        if state.cells.contains_key(&branch_id) || !state.reserved_targets.insert(branch_id.clone())
        {
            return Err(RelationalForkTargetReservationDenial::Duplicate);
        }
        Ok(RelationalForkTargetReservation {
            registry: self.clone(),
            branch_id,
            active: true,
        })
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, RelationalBranchRegistryState> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, RelationalBranchRegistryState> {
        self.state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RelationalForkTargetReservation {
    /// Exact destination name reserved by the issuing Relational owner.
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) fn install(mut self, cell: RelationalBranchReferenceCell) {
        let mut state = self.registry.write();
        assert_eq!(cell.identity().branch_id(), &self.branch_id);
        assert!(state.reserved_targets.remove(&self.branch_id));
        assert!(state.cells.insert(self.branch_id.clone(), cell).is_none());
        self.active = false;
    }
}

impl Drop for RelationalForkTargetReservation {
    fn drop(&mut self) {
        if self.active {
            self.registry
                .write()
                .reserved_targets
                .remove(&self.branch_id);
        }
    }
}
