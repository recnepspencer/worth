//! Actual conventional Source decisions and their issuer-owned principal support.
use super::WorthQueryRetainedAuthorizationDecisionFacts;
mod meaning;
pub(in crate::domain_computation) use meaning::SourceAuthorizationLocus;
use std::sync::Arc;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};
use {
    crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry,
    crate::domain_computation::primary_graph::SourceFactsAllocation,
    crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
};

pub(in crate::domain_computation) enum SourceReadSupportCurrentnessStop {
    PrincipalStale,
    AuthorizationStale,
    Admission(CompanionPreflightStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    PrincipalUnavailable,
    PrincipalWorkExhausted,
}

pub(in crate::domain_computation) struct SourceReadSupport {
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    registry: Arc<WorthQueryInstalledAuthorizationRegistry>,
    principal_facts: Vec<WorthQueryApplicationObservedFact>,
    _allocation: Arc<SourceFactsAllocation>,
}

impl SourceReadSupport {
    pub(in crate::domain_computation) fn retain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Self, CompanionPreflightStop> {
        let principal_facts = authorization.source_query_principal_facts(|bytes| {
            let bytes = u64::try_from(bytes)
                .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            allocation
                .claim(execution, bytes)
                .map_err(CompanionPreflightStop::Execution)?;
            SourceFactsAllocation::work(execution, bytes).map_err(CompanionPreflightStop::Execution)
        })?;
        Ok(Self {
            authorization,
            registry: Arc::clone(registry),
            principal_facts,
            _allocation: Arc::clone(allocation),
        })
    }

    pub(in crate::domain_computation) fn belongs_to_session(
        &self,
        session: WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.authorization.belongs_to_session(session)
    }

    pub(in crate::domain_computation) fn principal_facts(
        &self,
    ) -> &[WorthQueryApplicationObservedFact] {
        &self.principal_facts
    }

    pub(in crate::domain_computation) fn validate_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<(), SourceReadSupportCurrentnessStop> {
        use super::AuthorizationCurrentnessStop;
        use SourceReadSupportCurrentnessStop as Stop;
        crate::domain_computation::primary_graph::validate_source_principal_facts(
            &self.principal_facts,
            execution,
            runtime,
            snapshot,
        )?;
        let current = self
            .authorization
            .conventional_decisions_current_admitted(
                execution,
                runtime,
                snapshot,
                self.registry.bridge(),
            )
            .map_err(|stop| match stop {
                AuthorizationCurrentnessStop::Admission(stop) => Stop::Admission(stop),
                AuthorizationCurrentnessStop::Execution(stop) => Stop::Execution(stop),
            })?;
        current.then_some(()).ok_or(Stop::AuthorizationStale)
    }
}
