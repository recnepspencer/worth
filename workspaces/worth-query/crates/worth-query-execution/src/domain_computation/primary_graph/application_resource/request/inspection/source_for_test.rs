//! Closed certification reads compare actual Native source, never transport fields.
use super::*;
use crate::domain_computation::primary_graph::application_resource::occurrence::ResourceLifecycle;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    /// Instrumentation using actual public State admission before source comparison.
    pub fn plain_source_matches_read_for_test<Binding>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        expected: &WorthQueryApplicationReadObservation,
    ) -> Result<(bool, u64, u64), WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
    {
        self.inspect(
            selector,
            Purpose::State,
            |_, _, _| Ok(None),
            |execution, row| self.compare_source_read(execution, row, expected),
        )
    }

    /// Instrumentation only. The tuple reports actual equality followed by the
    /// expected read's Native commit and branch version; none grants authority.
    pub fn source_matches_read_for_test<Binding, Capability, Operation, Input>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        expected: &WorthQueryApplicationReadObservation,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<(bool, u64, u64), WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::State,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |execution, row| self.compare_source_read(execution, row, expected),
        )
    }

    fn compare_source_read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        row: &ResourceRowLease,
        expected: &WorthQueryApplicationReadObservation,
    ) -> Result<(bool, u64, u64), ResourceCause> {
        let expected = self
            .runtime
            .select_application_read_observation_with_request(execution, expected)
            .map_err(ResourceCause::Selection)?;
        let observed = expected.product().relational_basis().observation();
        let commit = observed.commit_id().ok_or(ResourceCause::Unavailable)?;
        let ResourceLifecycle::Live { owner, .. } = &row.occurrence().lifecycle else {
            return Err(ResourceCause::Unavailable);
        };
        let actual = owner
            .request()
            .ok_or(ResourceCause::Unavailable)?
            .request()
            .basis_binding()
            .truth_view_basis()
            .truth_commit_identity();
        Ok((
            actual
                == Some(
                    &worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                        commit.0,
                    ),
                ),
            commit.0,
            observed.truth_version().as_u64(),
        ))
    }
}
