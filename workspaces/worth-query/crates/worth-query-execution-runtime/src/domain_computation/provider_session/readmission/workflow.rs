#![deny(private_interfaces)]

use super::super::{
    WorthQueryExecutionAttemptIdentity, WorthQueryExecutionProviderSession,
    WorthQueryExecutionResourceAttemptEvidence, WorthQueryWorkflowExecutionResourceAttempt,
};
use std::sync::Arc;
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;

use crate::domain_computation::provider_session::graph_provider::{
    WorthQueryGraphProviderCall, WorthQueryGraphProviderCallReadmissionPlan,
};

pub struct WorthQueryWorkflowResourceReadmissionPreProvider {
    yielded_attempt: WorthQueryWorkflowExecutionResourceAttempt,
    fresh_attempt_identity: WorthQueryExecutionAttemptIdentity,
    fresh_provider_session: WorthQueryExecutionProviderSession,
    fresh_evidence: WorthQueryExecutionResourceAttemptEvidence,
    fresh_call: WorthQueryGraphProviderCall,
}

pub struct WorthQueryWorkflowResourceReadmissionPostProvider {
    yielded_attempt: WorthQueryWorkflowExecutionResourceAttempt,
    fresh_attempt_identity: WorthQueryExecutionAttemptIdentity,
    fresh_provider_session: WorthQueryExecutionProviderSession,
    fresh_evidence: WorthQueryExecutionResourceAttemptEvidence,
}

pub struct WorthQueryWorkflowProviderWorkRebinding {
    yielded: super::super::WorthQueryExecutionProviderSessionIdentity,
    fresh: super::super::WorthQueryExecutionProviderSessionIdentity,
}

impl WorthQueryWorkflowResourceReadmissionPreProvider {
    pub fn begin(
        yielded_attempt: WorthQueryWorkflowExecutionResourceAttempt,
        stage_resources: Arc<WorthQueryAdmittedExecutionResourcePlan>,
        call: WorthQueryGraphProviderCallReadmissionPlan,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> Self {
        let fresh_attempt_identity = WorthQueryExecutionAttemptIdentity::mint();
        let fresh_provider_session = WorthQueryExecutionProviderSession::mint(
            &fresh_attempt_identity,
            yielded_attempt.binding_authority(),
        );
        let fresh_evidence = WorthQueryExecutionResourceAttemptEvidence::capture(
            yielded_attempt.operation_resources(),
            &fresh_provider_session,
        );
        let stage_evidence = WorthQueryExecutionResourceAttemptEvidence::capture(
            &stage_resources,
            &fresh_provider_session,
        );
        let fresh_call = call.mint(&fresh_provider_session, &stage_evidence);
        Self {
            yielded_attempt,
            fresh_attempt_identity,
            fresh_provider_session,
            fresh_evidence,
            fresh_call,
        }
    }

    pub fn attempt_identity(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> &WorthQueryExecutionAttemptIdentity {
        &self.fresh_attempt_identity
    }

    pub fn extract_provider_call(
        self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> (
        WorthQueryWorkflowResourceReadmissionPostProvider,
        WorthQueryGraphProviderCall,
    ) {
        (
            WorthQueryWorkflowResourceReadmissionPostProvider {
                yielded_attempt: self.yielded_attempt,
                fresh_attempt_identity: self.fresh_attempt_identity,
                fresh_provider_session: self.fresh_provider_session,
                fresh_evidence: self.fresh_evidence,
            },
            self.fresh_call,
        )
    }

    pub fn yielded_binding_authority(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> &crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority
    {
        self.yielded_attempt.binding_authority()
    }

    pub fn abort(
        self,
        _owner: crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> WorthQueryWorkflowExecutionResourceAttempt {
        self.yielded_attempt
    }
}

impl WorthQueryWorkflowResourceReadmissionPostProvider {
    pub fn abort(
        self,
        _owner: crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> WorthQueryWorkflowExecutionResourceAttempt {
        self.yielded_attempt
    }

    pub fn commit(
        self,
        _owner: crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> WorthQueryWorkflowExecutionResourceAttempt {
        let Self {
            yielded_attempt,
            fresh_attempt_identity,
            fresh_provider_session,
            fresh_evidence,
        } = self;
        let WorthQueryWorkflowExecutionResourceAttempt {
            mut reserved,
            attempt_identity: _,
            provider_session,
            evidence: _,
            artifact_run,
        } = yielded_attempt;
        drop(provider_session);
        reserved.resources_mut().record_provider_session_mint();
        WorthQueryWorkflowExecutionResourceAttempt {
            reserved,
            attempt_identity: fresh_attempt_identity,
            provider_session: fresh_provider_session,
            evidence: fresh_evidence,
            artifact_run,
        }
    }

    pub fn provider_work_rebinding(
        &self,
        _owner: &crate::domain_computation::managed_run::WorthQueryWorkflowRunTransitionPermit,
    ) -> WorthQueryWorkflowProviderWorkRebinding {
        WorthQueryWorkflowProviderWorkRebinding {
            yielded: self.yielded_attempt.provider_session.closed_identity(),
            fresh: self.fresh_provider_session.closed_identity(),
        }
    }
}

impl WorthQueryWorkflowProviderWorkRebinding {
    pub fn admits(
        &self,
        current: &super::super::WorthQueryExecutionProviderSessionIdentity,
    ) -> bool {
        self.yielded == *current
    }

    pub fn into_fresh(self) -> super::super::WorthQueryExecutionProviderSessionIdentity {
        self.fresh
    }
}
