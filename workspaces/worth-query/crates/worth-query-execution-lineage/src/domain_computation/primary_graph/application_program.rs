//! Move-only owner evidence for one performed source publication.

pub struct WorthQueryPreparedRequiredOutputSource {
    pub(crate) runtime_authority: u64,
    pub(crate) source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    pub(crate) product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub(crate) owner: super::application_output_demand::WorthQueryOutputDemandRegistry,
}

impl Drop for WorthQueryPreparedRequiredOutputSource {
    fn drop(&mut self) {
        self.owner.release_prepared_token(&self.source_commit);
    }
}

/// Crate-boundary internal port for `WorthQueryPreparedRequiredOutputSource`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait PreparedRequiredOutputSourceInternalPort: Sized {
    fn from_parts(
        runtime_authority: u64,
        source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        owner: super::application_output_demand::WorthQueryOutputDemandRegistry,
    ) -> Self;
    fn runtime_authority(&self) -> u64;
    fn source_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity;
    fn product_occurrence(&self) -> worth_runtime_world::facade::ProductBranchIncarnation;
}

impl PreparedRequiredOutputSourceInternalPort for WorthQueryPreparedRequiredOutputSource {
    fn from_parts(
        runtime_authority: u64,
        source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        owner: super::application_output_demand::WorthQueryOutputDemandRegistry,
    ) -> Self {
        Self {
            runtime_authority,
            source_commit,
            product_occurrence,
            owner,
        }
    }

    fn runtime_authority(&self) -> u64 {
        self.runtime_authority
    }

    fn source_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        &self.source_commit
    }

    fn product_occurrence(&self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.product_occurrence
    }
}
