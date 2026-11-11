//! Query receipts for the actual complete original fate, without a new branch.
#[derive(Debug)]
pub struct WorthQueryProductBranchCreationNoEffect {
    original: worth_runtime_world::facade::RuntimeWorldBranchCreationNoEffect,
}
impl WorthQueryProductBranchCreationNoEffect {
    pub(crate) fn from_world(
        original: worth_runtime_world::facade::RuntimeWorldBranchCreationNoEffect,
    ) -> Self {
        Self { original }
    }
    pub fn is_complete(&self) -> bool {
        let _original = self.original.association();
        true
    }
}
#[derive(Debug)]
#[must_use = "an original release is distinct from pre-admission denial"]
pub struct WorthQueryProductBranchCreationRelease {
    original: worth_runtime_world::facade::RuntimeWorldBranchCreationRelease,
}
impl WorthQueryProductBranchCreationRelease {
    pub(crate) fn from_world(
        original: worth_runtime_world::facade::RuntimeWorldBranchCreationRelease,
    ) -> Self {
        Self { original }
    }
    pub fn is_complete(&self) -> bool {
        let _original = self.original.association();
        true
    }
}
