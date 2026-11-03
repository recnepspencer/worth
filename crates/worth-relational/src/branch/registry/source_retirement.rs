//! Dense stopped-owner cells lend their actual roots without copying names.
use super::*;

pub(crate) enum RelationalSourceRootSlot {
    Root(Arc<super::super::RelationalBranchRoot>),
    Empty,
    Complete,
}
impl RelationalBranchReferenceRegistry {
    pub(crate) fn source_retirement_root(
        &self,
        index: usize,
    ) -> Result<RelationalSourceRootSlot, ()> {
        let registry = match self.state.try_read() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        let Some((_, cell)) = registry.cells.get_index(index) else {
            return Ok(RelationalSourceRootSlot::Complete);
        };
        let state = match cell.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        Ok(match &state.root {
            Some(root) => RelationalSourceRootSlot::Root(root.clone()),
            None => RelationalSourceRootSlot::Empty,
        })
    }
}
