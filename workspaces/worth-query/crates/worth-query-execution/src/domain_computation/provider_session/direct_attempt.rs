use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_query_admission::integration::{
    WorthQueryCapacityReservedExecutionResourcePlan, WorthQueryExecutionCapacityReleaseReceipt,
};

use super::{
    WorthQueryExecutionAttemptIdentity, WorthQueryExecutionProviderSession,
    WorthQueryExecutionResourceAttemptEvidence,
};

mod readmission;

pub(in crate::domain_computation) use readmission::{
    WorthQueryDirectProviderWorkRebinding, WorthQueryDirectResourceReadmissionPending,
};
pub struct WorthQueryDirectExecutionResourceAttempt {
    reserved: WorthQueryCapacityReservedExecutionResourcePlan,
    attempt_identity: WorthQueryExecutionAttemptIdentity,
    provider_session: WorthQueryExecutionProviderSession,
    evidence: WorthQueryExecutionResourceAttemptEvidence,
}

impl WorthQueryDirectExecutionResourceAttempt {
    pub(crate) fn start(
        mut reserved: WorthQueryCapacityReservedExecutionResourcePlan,
        binding_authority: &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority,
    ) -> Self {
        let attempt_identity = WorthQueryExecutionAttemptIdentity::mint();
        let provider_session =
            WorthQueryExecutionProviderSession::mint(&attempt_identity, binding_authority);
        reserved.resources_mut().record_provider_session_mint();
        let evidence = WorthQueryExecutionResourceAttemptEvidence::capture(
            reserved.resources(),
            &provider_session,
        );
        Self {
            reserved,
            attempt_identity,
            provider_session,
            evidence,
        }
    }

    pub fn resources(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        self.reserved.resources()
    }

    pub(in crate::domain_computation) fn managed_provider_session(
        &self,
    ) -> &WorthQueryExecutionProviderSession {
        &self.provider_session
    }

    #[cfg(test)]
    pub(crate) fn provider_session_for_test(&self) -> &WorthQueryExecutionProviderSession {
        &self.provider_session
    }

    pub fn evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        &self.evidence
    }

    pub fn attempt_identity(&self) -> &WorthQueryExecutionAttemptIdentity {
        &self.attempt_identity
    }

    pub(crate) fn retained_capacity_reservation_count(&self) -> usize {
        self.reserved.reservation_count()
    }

    pub(crate) fn binding_authority(
        &self,
    ) -> &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority
    {
        self.provider_session.binding_authority()
    }

    pub(crate) fn release(self) -> WorthQueryDirectExecutionAttemptReleaseReceipt {
        let provider_session_identity = self.provider_session.identity().to_owned();
        drop(self.provider_session);
        WorthQueryDirectExecutionAttemptReleaseReceipt {
            provider_session_identity,
            capacity: self.reserved.release(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDirectExecutionAttemptReleaseReceipt {
    provider_session_identity: String,
    capacity: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryDirectExecutionAttemptReleaseReceipt {
    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn capacity(&self) -> &WorthQueryExecutionCapacityReleaseReceipt {
        &self.capacity
    }
}
