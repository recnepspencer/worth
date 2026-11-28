//! Dense stopped-owner cells lend their actual roots without copying names.
use super::*;
use crate::history::retention::RetentionBusy;

pub enum RelationalSourceRootSlot {
    Root(Arc<crate::branch::root::RelationalBranchRoot>),
    Empty,
    Complete,
}
impl RelationalBranchReferenceRegistry {
    pub fn source_retirement_root(
        &self,
        index: usize,
    ) -> Result<RelationalSourceRootSlot, RetentionBusy> {
        let registry = worth_execution::try_retirement_guard(self.state.try_read(), RetentionBusy)?;
        let Some((_, cell)) = registry.cells.get_index(index) else {
            return Ok(RelationalSourceRootSlot::Complete);
        };
        let state = worth_execution::try_retirement_guard(cell.state.try_lock(), RetentionBusy)?;
        Ok(match &state.root {
            Some(root) => RelationalSourceRootSlot::Root(root.clone()),
            None => RelationalSourceRootSlot::Empty,
        })
    }
}
