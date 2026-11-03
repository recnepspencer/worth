//! One installed provider consumes the two actual product-retention notifications.
/// The installed provider follows actual product-history and Native owner retirement.
pub(crate) trait WorthQueryProductRetentionFollower: Send + Sync {
    /// History a walk released may have been the last to keep evidence Query
    /// inherited onto its commits; that evidence goes now.
    fn history_released(&self);
    /// Immutable installed capability; absence of a live lookup is not stable.
    fn manages_native_sources(
        &self,
        identity: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> bool;
    /// Consumes actual Native completion after every enclosing cleanup guard.
    fn native_branch_deleted(
        &self,
        deleted: &worth_relational::facade::branch::DeletedRelationalBranch,
    );
}
