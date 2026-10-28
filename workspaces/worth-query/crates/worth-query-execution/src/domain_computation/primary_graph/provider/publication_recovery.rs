use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn repair_equivalent_publication_settlement(
        &self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        committed: &super::WorthQueryPrimaryGraphCommittedApplication,
    ) -> Result<(), super::WorthQueryProviderIdempotencyResolutionDenial> {
        runtime
            .repair_pending_publication_settlement(committed.commit_reference().commit_id)
            .map(|_| ())
            .map_err(|_| super::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)
    }
}
