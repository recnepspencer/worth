//! Each writer retains its own typed authorization at the shared effect boundary.
use super::*;
impl CommitPreparedCoupledWriter<'_> {
    pub(in crate::domain_computation::primary_graph) fn authorize<S, B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        let typed = self
            .custody
            .typed
            .downcast_ref::<CoupledWriterAdmission<S, B>>()
            .expect("installed executor retains its own nominal writer admission");
        let proof = typed.authorization.authorize_application_commit(
            execution,
            application,
            &typed.admission,
            coordination,
        )?;
        proof.govern((), |()| ()).map_err(|((), denial)| denial)
    }
}
