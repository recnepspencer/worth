//! Move-only authorization transition from provider registration to commit.

use super::{WorthQueryCommitAuthorizationBasis, WorthQueryProviderAuthorizationDecisionFacts};

pub struct WorthQueryProviderCommitAuthorization {
    provider: Option<WorthQueryProviderAuthorizationDecisionFacts>,
    commit: WorthQueryCommitAuthorizationBasis,
    workflow_approval_authority: Option<super::super::WorthQueryWorkflowApprovalAuthorityBasis>,
}

impl WorthQueryProviderCommitAuthorization {
    pub fn new(
        provider: WorthQueryProviderAuthorizationDecisionFacts,
        commit: WorthQueryCommitAuthorizationBasis,
    ) -> Self {
        Self {
            provider: Some(provider),
            commit,
            workflow_approval_authority: None,
        }
    }

    pub fn with_workflow_approval_authority(
        mut self,
        authority: Option<super::super::WorthQueryWorkflowApprovalAuthorityBasis>,
    ) -> Self {
        self.workflow_approval_authority = authority;
        self
    }

    pub fn commit_basis(&self) -> &WorthQueryCommitAuthorizationBasis {
        &self.commit
    }

    pub fn workflow_approval_authority(
        &self,
    ) -> Option<&super::super::WorthQueryWorkflowApprovalAuthorityBasis> {
        self.workflow_approval_authority.as_ref()
    }

    pub fn register_provider_attempt<'run, Schema, Operation, Input, Scope>(
        &mut self,
        prepared: crate::domain_computation::primary_graph::WorthQueryPreparedApplicationProviderAttempt,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        attempt_basis: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptBasis,
        context: crate::domain_computation::primary_graph::WorthQueryProviderAttemptRegistrationContext<'_, Schema, Operation, Input, Scope>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryRegisteredProviderAttempt<'run>,
        crate::domain_computation::primary_graph::WorthQueryProviderProgressionOutcome,
    > {
        let Some(provider) = self.provider.take() else {
            let _ = staged.abort();
            return Err(crate::domain_computation::primary_graph::progression_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            ));
        };
        prepared.register(staged, provider, attempt_basis, context, allocation_policy)
    }

    pub fn finish_registration(self) -> Option<WorthQueryRegisteredCommitAuthorization> {
        self.provider
            .is_none()
            .then_some(WorthQueryRegisteredCommitAuthorization {
                authorization: self,
            })
    }
}

pub struct WorthQueryRegisteredCommitAuthorization {
    authorization: WorthQueryProviderCommitAuthorization,
}

impl WorthQueryRegisteredCommitAuthorization {
    pub fn authorization(&self) -> &WorthQueryProviderCommitAuthorization {
        &self.authorization
    }
}

impl WorthQueryProviderCommitAuthorization {
    pub fn authorize_application_commit<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &impl crate::ports::CommitAuthorizationRuntimePort<Schema>,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        serialization: &'serialization crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        application.authorize_with_commit_basis(
            execution,
            self.workflow_approval_authority(),
            self.commit_basis(),
            admission,
            serialization,
        )
    }

    #[doc(hidden)]
    pub fn authorize_application_commit_for_test<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &impl crate::ports::CommitAuthorizationRuntimePort<Schema>,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        serialization: &'serialization crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        self.authorize_application_commit(execution, application, admission, serialization)
    }
}

impl WorthQueryRegisteredCommitAuthorization {
    pub fn authorize_application_commit<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &impl crate::ports::CommitAuthorizationRuntimePort<Schema>,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        serialization: &'serialization crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        self.authorization().authorize_application_commit(
            execution,
            application,
            admission,
            serialization,
        )
    }
}
