use super::super::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionAttemptIdentity,
    WorthQueryExecutionProviderSession, WorthQueryExecutionResourceAttemptEvidence,
};
use crate::domain_computation::provider_session::graph_provider::{
    WorthQueryGraphProviderCall, WorthQueryGraphProviderCallReadmissionPlan,
};

pub(crate) struct WorthQueryDirectResourceReadmissionPending {
    yielded_attempt: WorthQueryDirectExecutionResourceAttempt,
    fresh_attempt_identity: WorthQueryExecutionAttemptIdentity,
    fresh_provider_session: WorthQueryExecutionProviderSession,
    fresh_evidence: WorthQueryExecutionResourceAttemptEvidence,
}

pub(in crate::domain_computation) struct WorthQueryDirectProviderWorkRebinding {
    yielded: super::super::WorthQueryExecutionProviderSessionIdentity,
    fresh: super::super::WorthQueryExecutionProviderSessionIdentity,
}

impl WorthQueryDirectResourceReadmissionPending {
    pub(crate) fn begin(
        yielded_attempt: WorthQueryDirectExecutionResourceAttempt,
        call: WorthQueryGraphProviderCallReadmissionPlan,
        _owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> (Self, WorthQueryGraphProviderCall) {
        let fresh_attempt_identity = WorthQueryExecutionAttemptIdentity::mint();
        let fresh_provider_session = WorthQueryExecutionProviderSession::mint(
            &fresh_attempt_identity,
            yielded_attempt.binding_authority(),
        );
        let fresh_evidence = WorthQueryExecutionResourceAttemptEvidence::capture(
            yielded_attempt.resources(),
            &fresh_provider_session,
        );
        let fresh_call = call.mint(&fresh_provider_session, &fresh_evidence);
        (
            Self {
                yielded_attempt,
                fresh_attempt_identity,
                fresh_provider_session,
                fresh_evidence,
            },
            fresh_call,
        )
    }

    pub(crate) fn attempt_identity(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> &WorthQueryExecutionAttemptIdentity {
        &self.fresh_attempt_identity
    }

    pub(crate) fn yielded_binding_authority(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority
    {
        self.yielded_attempt.binding_authority()
    }

    pub(crate) fn abort(
        self,
        _owner: crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectExecutionResourceAttempt {
        self.yielded_attempt
    }

    pub(crate) fn commit(
        self,
        _owner: crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectExecutionResourceAttempt {
        let Self {
            yielded_attempt,
            fresh_attempt_identity,
            fresh_provider_session,
            fresh_evidence,
        } = self;
        let WorthQueryDirectExecutionResourceAttempt {
            mut reserved,
            attempt_identity: _,
            provider_session,
            evidence: _,
        } = yielded_attempt;
        drop(provider_session);
        reserved.resources_mut().record_provider_session_mint();
        WorthQueryDirectExecutionResourceAttempt {
            reserved,
            attempt_identity: fresh_attempt_identity,
            provider_session: fresh_provider_session,
            evidence: fresh_evidence,
        }
    }

    pub(crate) fn provider_work_rebinding(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectProviderWorkRebinding {
        WorthQueryDirectProviderWorkRebinding {
            yielded: self
                .yielded_attempt
                .managed_provider_session()
                .closed_identity(),
            fresh: self.fresh_provider_session.closed_identity(),
        }
    }
}

impl WorthQueryDirectProviderWorkRebinding {
    pub(in crate::domain_computation) fn admits(
        &self,
        current: &super::super::WorthQueryExecutionProviderSessionIdentity,
    ) -> bool {
        self.yielded == *current
    }

    pub(in crate::domain_computation) fn into_fresh(
        self,
    ) -> super::super::WorthQueryExecutionProviderSessionIdentity {
        self.fresh
    }
}
