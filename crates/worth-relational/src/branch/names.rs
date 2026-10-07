use crate::history::data::BranchId;

/// One coherent read of registered branch names and permanently retired names.
/// Registered names include archived and deleting references; a deleting name
/// can also be retired. Neither list grants branch selection authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchNames {
    registered: Vec<BranchId>,
    retired: Vec<BranchId>,
}

impl RelationalBranchNames {
    pub(crate) fn new(mut registered: Vec<BranchId>, mut retired: Vec<BranchId>) -> Self {
        registered.sort();
        retired.sort();
        Self {
            registered,
            retired,
        }
    }

    pub fn registered(&self) -> &[BranchId] {
        &self.registered
    }

    pub fn retired(&self) -> &[BranchId] {
        &self.retired
    }
}
