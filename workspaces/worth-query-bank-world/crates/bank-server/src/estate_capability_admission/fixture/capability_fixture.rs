use bank_domain::estate::BankEstateWorld;
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use super::{block_on, request_scope, TestAuthenticationAdapter, TestCredential};
use crate::{BankAuthenticatedPrincipal, BankAuthenticationBoundary, BankIdentityRuntime};

pub(crate) struct CapabilityFixture {
    pub(crate) runtime: BankIdentityRuntime,
    pub(super) estate_world: BankEstateWorld,
    pub(super) authentication: BankAuthenticationBoundary<TestAuthenticationAdapter>,
    pub(super) specialist_identity: WorthQueryExternalPrincipalIdentity,
    pub(super) executor_identity: WorthQueryExternalPrincipalIdentity,
    pub(super) approver_identity: WorthQueryExternalPrincipalIdentity,
    pub(super) reviewer_identity: WorthQueryExternalPrincipalIdentity,
}

impl CapabilityFixture {
    pub(crate) fn authenticate(&self) -> BankAuthenticatedPrincipal {
        self.authenticate_identity(self.specialist_identity.clone())
    }

    pub(crate) fn authenticate_approver(&self) -> BankAuthenticatedPrincipal {
        self.authenticate_identity(self.approver_identity.clone())
    }

    pub(crate) fn authenticate_executor(&self) -> BankAuthenticatedPrincipal {
        self.authenticate_identity(self.executor_identity.clone())
    }

    pub(in crate::estate_capability_admission) fn authenticate_reviewer(
        &self,
    ) -> BankAuthenticatedPrincipal {
        self.authenticate_identity(self.reviewer_identity.clone())
    }

    fn authenticate_identity(
        &self,
        identity: WorthQueryExternalPrincipalIdentity,
    ) -> BankAuthenticatedPrincipal {
        let request = request_scope();
        block_on(self.runtime.authenticate_with(
            &self.authentication,
            TestCredential(identity),
            &request,
        ))
        .expect("the mapped employee should authenticate")
    }

    pub(in crate::estate_capability_admission) fn source_beneficiaries(
        &self,
    ) -> Vec<bank_domain::model::BankPrincipalId> {
        self.estate_world
            .beneficiaries()
            .filter_map(|(estate, principal)| (estate == super::ESTATE).then_some(principal))
            .collect()
    }
}

impl CapabilityFixture {
    pub(in crate::estate_capability_admission) fn lifecycle_original(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: bank_domain::estate::EstateAction,
        key: &bank_domain::proposals::BankIdempotencyKey,
        scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> worth_query_host::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt {
        use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
        match self
            .runtime
            .observe_estate_lifecycle_idempotency(principal, action, key, scope)
            .unwrap()
        {
            Resolution::RecoveredCommitted(receipt) => receipt,
            other @ (Resolution::Unseen
            | Resolution::IntentDrift
            | Resolution::ProductPreparationPending(_)) => {
                panic!("the original lifecycle key must resolve to data only: {other:?}")
            }
        }
    }
    pub(in crate::estate_capability_admission) fn lifecycle_head(
        &self,
        principal: &BankAuthenticatedPrincipal,
        scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> u64 {
        self.runtime
            .with_application_advancement(scope, |phase| {
                self.runtime.request(principal, scope).retain_read(&phase)
            })
            .unwrap()
            .unwrap()
            .selected_commit()
            .ordinal()
    }
}
