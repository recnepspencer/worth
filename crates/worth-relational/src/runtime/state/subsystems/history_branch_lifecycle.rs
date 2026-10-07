use crate::history::data::BranchId;

use super::HistorySubsystem;

const MAX_RETIRED_BRANCH_NAMES: usize = 65_536;

impl HistorySubsystem {
    pub(crate) fn branch_names(&self) -> crate::branch::RelationalBranchNames {
        self.branch_cells.branch_names()
    }

    pub(crate) fn remove_branch_cell(
        &self,
        branch_id: &BranchId,
    ) -> Option<crate::branch::RelationalBranchReferenceCell> {
        self.branch_cells.remove(branch_id)
    }

    pub(crate) fn reserve_branch_name_retirement(&self, branch_id: BranchId) -> Result<(), ()> {
        self.branch_cells
            .reserve_name_retirement(branch_id, MAX_RETIRED_BRANCH_NAMES)
    }

    pub(crate) fn retired_branch_names_checkpoint(&self) -> Vec<BranchId> {
        self.branch_cells.retired_names_checkpoint()
    }

    /// Install the branch cells and retired names of one durable checkpoint.
    pub(super) fn restore_branch_registry(
        &self,
        cells: std::collections::BTreeMap<BranchId, crate::branch::RelationalBranchReferenceCell>,
        retired_names: &[BranchId],
    ) -> Result<(), String> {
        self.branch_cells.restore_checkpoint(
            cells,
            retired_names,
            MAX_RETIRED_BRANCH_NAMES,
            &self.main_branch,
        )
    }

    #[cfg(test)]
    fn fill_retired_branch_name_capacity_for_test(&self) {
        let mut ordinal = 0_u64;
        loop {
            if self
                .branch_cells
                .reserve_name_retirement(
                    BranchId(format!("__retired-branch-capacity-proof-{ordinal}")),
                    MAX_RETIRED_BRANCH_NAMES,
                )
                .is_err()
            {
                break;
            }
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
