//! Move-only authorization transition from provider registration to commit.

use super::{WorthQueryCommitAuthorizationBasis, WorthQueryProviderAuthorizationDecisionFacts};

pub(in crate::domain_computation) struct WorthQueryProviderCommitAuthorization {
    provider: Option<WorthQueryProviderAuthorizationDecisionFacts>,
    commit: WorthQueryCommitAuthorizationBasis,
}

impl WorthQueryProviderCommitAuthorization {
    pub(in crate::domain_computation::authorization) fn new(
        provider: WorthQueryProviderAuthorizationDecisionFacts,
        commit: WorthQueryCommitAuthorizationBasis,
    ) -> Self {
        Self {
            provider: Some(provider),
            commit,
        }
    }

    pub(in crate::domain_computation) fn authorize_application_commit<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
        application.authorize_application_commit(admission, &self.commit, serialization)
    }

    pub(in crate::domain_computation) fn register_provider_attempt<
        'run,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &mut self,
        prepared: crate::domain_computation::primary_graph::WorthQueryPreparedApplicationProviderAttempt,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        attempt_basis: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptBasis,
        context: crate::domain_computation::primary_graph::WorthQueryProviderAttemptRegistrationContext<'_, Schema, Operation, Input, Scope>,
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
        prepared.register(staged, provider, attempt_basis, context)
    }

    pub(in crate::domain_computation) fn finish_registration(
        self,
    ) -> Option<WorthQueryRegisteredCommitAuthorization> {
        self.provider
            .is_none()
            .then_some(WorthQueryRegisteredCommitAuthorization {
                authorization: self,
            })
    }
}

pub(in crate::domain_computation) struct WorthQueryRegisteredCommitAuthorization {
    authorization: WorthQueryProviderCommitAuthorization,
}

impl WorthQueryRegisteredCommitAuthorization {
    pub(in crate::domain_computation) fn authorize_application_commit<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
        self.authorization
            .authorize_application_commit(application, admission, serialization)
    }
}
