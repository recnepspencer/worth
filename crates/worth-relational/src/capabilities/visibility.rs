use crate::runtime::RelationalRuntime;

pub(crate) trait VisibilityPolicySource {
    fn visibility_cache_enabled(&self) -> bool;
    fn recent_visibility_window(&self) -> usize;
    fn protect_active_snapshots(&self) -> bool;
    fn protect_branch_heads(&self) -> bool;
}

impl VisibilityPolicySource for RelationalRuntime {
    fn visibility_cache_enabled(&self) -> bool {
        self.config.visibility.cache_policy.enabled
    }

    fn recent_visibility_window(&self) -> usize {
        self.config.visibility.cache_policy.recent_version_window
    }

    fn protect_active_snapshots(&self) -> bool {
        self.config.visibility.cache_policy.protect_active_snapshots
    }

    fn protect_branch_heads(&self) -> bool {
        self.config.visibility.cache_policy.protect_branch_heads
    }
}

pub(crate) trait PublicationPolicySource {
    fn max_patch_records_per_commit(&self) -> Option<usize>;
}

impl PublicationPolicySource for RelationalRuntime {
    fn max_patch_records_per_commit(&self) -> Option<usize> {
        self.config.publication.policy.max_patch_records_per_commit
    }
}
