//! Move-only owner evidence for one performed source publication.

pub struct WorthQueryPreparedRequiredOutputSource {
    pub runtime_authority: u64,
    pub source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    pub product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub owner: super::application_output_demand::WorthQueryOutputDemandRegistry,
}

impl Drop for WorthQueryPreparedRequiredOutputSource {
    fn drop(&mut self) {
        self.owner.release_prepared_token(&self.source_commit);
    }
}
