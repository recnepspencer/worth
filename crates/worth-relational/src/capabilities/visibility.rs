use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn protect_branch_heads(&self) -> bool {
        self.config.visibility.cache_policy.protect_branch_heads
    }
}
