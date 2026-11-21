//! Native certifies the complete roster before ordinary relation laws judge deletion.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(super) fn validate_owned_member_rosters(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
        let staged = attempts
            .staged_attempt(session)
            .ok_or_else(missing_candidate_failure)?;
        let basis = staged
            .product_publication()
            .observation()
            .basis()
            .relational_basis();
        let rosters = staged.owned_member_rosters(&self.graph.layout)?;
        for (anchor, kinds, roles) in rosters {
            self.graph
                .with_runtime(execution, |runtime| {
                    runtime
                        .validate_authored_owned_members(execution, basis, anchor, &kinds, &roles)
                })
                .map_err(map_validation_failure)?;
        }
        Ok(())
    }
}
