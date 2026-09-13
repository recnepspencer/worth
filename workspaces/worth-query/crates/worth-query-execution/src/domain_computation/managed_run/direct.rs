use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::run_affinity::WorthQueryDirectRunAffinity;
use super::{
    WorthQueryDirectRunTerminal, WorthQueryManagedGraphCallRequest, WorthQueryManagedRunCounters,
    WorthQueryManagedRunDenial, WorthQueryManagedRunDenialKind, WorthQueryManagedRunTerminalKind,
    WorthQueryManagedSafePointFailure, WorthQueryManagedSafePointObservation,
};
use crate::domain_computation::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionResourceAttemptEvidence, WorthQueryGraphCallBindingDenial,
    WorthQueryGraphProviderCallRequest,
};

pub struct WorthQueryAdmittedDirectRun {
    affinity: WorthQueryDirectRunAffinity,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
    counters: WorthQueryManagedRunCounters,
}

impl WorthQueryAdmittedDirectRun {
    pub(in crate::domain_computation) fn new(
        _operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
        bridge_basis: BridgeBoundExecutionBasis,
        relational_basis: WorthQueryManagedRelationalObservation,
        counters: WorthQueryManagedRunCounters,
    ) -> Self {
        Self {
            affinity: WorthQueryDirectRunAffinity::initial(resource_attempt),
            bridge_basis,
            relational_basis,
            counters,
        }
    }

    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }

    pub(crate) fn belongs_to_operation(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        self.affinity.belongs_to_operation(operation)
    }

    pub fn start(self) -> WorthQueryRunningDirectRun {
        WorthQueryRunningDirectRun {
            affinity: self.affinity,
            bridge_basis: self.bridge_basis,
            relational_basis: self.relational_basis,
            counters: self.counters,
        }
    }
}

pub struct WorthQueryRunningDirectRun {
    pub(super) affinity: WorthQueryDirectRunAffinity,
    pub(super) bridge_basis: BridgeBoundExecutionBasis,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) counters: WorthQueryManagedRunCounters,
}

impl WorthQueryRunningDirectRun {
    pub fn resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan
    {
        self.affinity.provider_plan_resources().0
    }

    pub fn provider_session_identity(&self) -> &str {
        self.affinity.provider_session_description()
    }

    pub fn bind_commit_call(
        &self,
        graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
        request: crate::domain_computation::WorthQueryGraphCommitCallRequest,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphCommitCall,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        let (resources, evidence) = self.affinity.provider_plan_resources();
        self.affinity
            .provider_plan_session()
            .bind_graph_commit_call(
                graph_authorities,
                request,
                evidence,
                resources.shared_envelope(),
            )
    }

    pub fn abandon(self) -> WorthQueryDirectRunTerminal {
        self.terminal(WorthQueryManagedRunTerminalKind::Failed)
    }

    pub(crate) fn graph_work_affinity(
        &self,
    ) -> Option<crate::domain_computation::operation_binding::WorthQueryApplicationGraphWorkAffinity>
    {
        self.affinity.graph_work_affinity()
    }

    pub(crate) fn mutation_resource_release_expectation(&self) -> (&str, &str, usize) {
        let (resource_plan, reservation_count) = self.affinity.resource_release_expectation();
        (
            self.affinity.provider_session_description(),
            resource_plan,
            reservation_count,
        )
    }

    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        self.affinity.evidence()
    }

    pub fn observe_safe_point(
        &self,
    ) -> Result<WorthQueryManagedSafePointObservation, WorthQueryManagedSafePointFailure> {
        self.affinity.observe_safe_point(&self.bridge_basis)
    }

    pub fn begin_graph_execution(
        self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<
        super::WorthQueryActiveDirectGraphExecution,
        super::WorthQueryDirectGraphExecutionStartFailure,
    > {
        super::direct_graph_execution_start::begin(self, graph_authority, request)
    }

    pub(super) fn mint_graph_provider_call(
        &self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphProviderCall,
        WorthQueryGraphCallBindingDenial,
    > {
        let request =
            WorthQueryGraphProviderCallRequest::direct(request.kind(), request.scope_identity())
                .with_managed_execution_snapshot(self.execution_snapshot_reference());
        self.affinity
            .bind_graph_provider_call(graph_authority, request)
    }

    pub(crate) fn execution_snapshot_reference(&self) -> String {
        let parts = self
            .bridge_basis
            .observation()
            .snapshot_identity()
            .relational_snapshot_parts()
            .expect("managed Relational run admission validates typed snapshot identity");
        format!(
            "worth-query-managed-snapshot|runtime={}|snapshot={}|version={}",
            self.relational_basis.identity().runtime_instance_id(),
            parts.snapshot_id(),
            parts.version_id(),
        )
    }

    pub fn completed(
        self,
    ) -> Result<WorthQueryDirectRunTerminal, WorthQueryDirectRunCompletionRejection> {
        if self.affinity.provider_work_has_uncertainty() {
            return Err(WorthQueryDirectRunCompletionRejection {
                denial: WorthQueryManagedRunDenial::new(
                    WorthQueryManagedRunDenialKind::UnverifiedProviderWork,
                    "provider work must be receipt-bound before a managed run can claim completion",
                    self.counters.clone(),
                ),
                running: self,
            });
        }
        Ok(self.terminal(WorthQueryManagedRunTerminalKind::Completed))
    }

    pub(super) fn provider_work_mut(
        &mut self,
    ) -> &mut super::provider_work::WorthQueryManagedProviderWorkLedger {
        self.affinity.provider_work_mut()
    }

    pub(super) fn graph_resource_support(
        &self,
        role: &str,
    ) -> Option<
        &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    > {
        self.affinity.graph_resource_support(role)
    }

    pub(super) fn bridge_basis(&self) -> &BridgeBoundExecutionBasis {
        &self.bridge_basis
    }

    pub(super) fn bridge_basis_mut(&mut self) -> &mut BridgeBoundExecutionBasis {
        &mut self.bridge_basis
    }

    pub(super) fn retained_capacity_reservation_count(&self) -> usize {
        self.affinity.retained_capacity_reservation_count()
    }

    pub(super) fn installation_is_current(&self) -> bool {
        self.affinity.installation_is_current()
    }

    pub(super) fn yield_is_installed(&self) -> bool {
        self.affinity.yield_is_installed()
    }

    pub(super) fn terminal(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectRunTerminal {
        let (affinity, provider_work, provider_cleanup) = self.affinity.into_terminal_parts();
        WorthQueryDirectRunTerminal {
            affinity,
            kind,
            bridge_basis: self.bridge_basis,
            relational_basis: self.relational_basis,
            counters: self.counters,
            provider_work,
            provider_cleanup,
        }
    }

    pub(crate) fn terminate_for_convergence(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectRunTerminal {
        self.terminal(kind)
    }
}

pub struct WorthQueryDirectRunCompletionRejection {
    denial: WorthQueryManagedRunDenial,
    running: WorthQueryRunningDirectRun,
}

impl WorthQueryDirectRunCompletionRejection {
    pub fn denial(&self) -> &WorthQueryManagedRunDenial {
        &self.denial
    }

    pub fn into_running(self) -> WorthQueryRunningDirectRun {
        self.running
    }
}

impl std::fmt::Debug for WorthQueryDirectRunCompletionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDirectRunCompletionRejection")
            .field("denial", &self.denial)
            .field("run_identity", &self.running.identity())
            .finish()
    }
}
