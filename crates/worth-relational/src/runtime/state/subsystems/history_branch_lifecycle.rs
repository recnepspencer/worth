use crate::history::data::BranchId;

use super::HistorySubsystem;

const MAX_RETIRED_BRANCH_NAMES: usize = 65_536;

impl HistorySubsystem {
    pub(crate) fn remove_branch_cell(
        &self,
        branch_id: &BranchId,
    ) -> Option<crate::branch::RelationalBranchReferenceCell> {
        self.branch_cells.remove(branch_id)
    }

    /// Whether retiring `branch_id` fits the bounded retired-name set. A
    /// deletion checks this inside the patch-position reservation, where only
    /// branch transitions retire names, so its cutover's retirement cannot
    /// fail.
    pub(crate) fn admits_branch_name_retirement(&self, branch_id: &BranchId) -> bool {
        self.branch_cells
            .admits_name_retirement(branch_id, MAX_RETIRED_BRANCH_NAMES)
    }

    /// Retire `branch_id` after `admits_branch_name_retirement` said it fits.
    pub(crate) fn retire_admitted_branch_name(&self, branch_id: BranchId) {
        self.branch_cells.retire_name(branch_id);
    }

    /// Every branch's lifecycle posture, in branch order, for a checkpoint.
    pub(crate) fn branch_lifecycle_snapshot(
        &self,
    ) -> Vec<crate::durability::data::DurableBranchLifecycleImage> {
        self.branch_cells.lifecycle_postures()
    }

    /// Every retired branch name, sorted, for a checkpoint.
    pub(crate) fn retired_branch_names(&self) -> Vec<BranchId> {
        self.branch_cells.retired_names()
    }

    /// Restore the retired names a checkpoint recorded, refusing a set the
    /// bound cannot hold or a name a recovered branch still uses.
    pub(crate) fn restore_retired_branch_names(&self, names: &[BranchId]) -> Result<(), String> {
        if names.len() > MAX_RETIRED_BRANCH_NAMES {
            return Err(format!(
                "checkpoint retires {} branch names; at most {MAX_RETIRED_BRANCH_NAMES} are held",
                names.len()
            ));
        }
        self.branch_cells.restore_retired_names(names)
    }

    /// Admit a transition before taking branch coordination or publication state.
    pub(crate) fn begin_branch_transition(&self) -> super::BranchTransitionPublication<'_> {
        self.canonical_publication_routes.begin_branch_transition()
    }

    #[cfg(test)]
    fn fill_retired_branch_name_capacity_for_test(&self) {
        let mut ordinal = 0_u64;
        loop {
            let name = BranchId(format!("__retired-branch-capacity-proof-{ordinal}"));
            if !self.admits_branch_name_retirement(&name) {
                break;
            }
            self.retire_admitted_branch_name(name);
            ordinal = ordinal
                .checked_add(1)
                .expect("retired branch capacity proof ordinal remains bounded");
        }
    }
}

#[cfg(test)]
impl crate::runtime::RelationalRuntime {
    pub(crate) fn fill_retired_branch_name_capacity_for_test(&self) {
        self.history.fill_retired_branch_name_capacity_for_test();
    }
}
