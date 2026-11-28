use crate::runtime::RelationalRuntime;

pub(crate) trait VisibilityPolicySource {
    fn protect_branch_heads(&self) -> bool;
}

impl VisibilityPolicySource for RelationalRuntime {
    fn protect_branch_heads(&self) -> bool {
        self.config.visibility.cache_policy.protect_branch_heads
    }
}
