use std::sync::Arc;

use super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::run_affinity::WorthQueryDirectRunAffinity;
use super::WorthQueryManagedRelationalObservation;
use super::{
    WorthQueryManagedRunCounters, WorthQueryPausedDirectGraphExecution,
    WorthQueryYieldTransitionCounters,
};
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDirectYieldDenialKind {
    InstallationGenerationStale,
    YieldNotInstalled,
    CheckpointUnavailable,
}

pub struct WorthQueryDirectYieldDenied {
    pub(super) kind: WorthQueryDirectYieldDenialKind,
    pub(super) detail: Arc<str>,
    pub(super) paused: WorthQueryPausedDirectGraphExecution,
    pub(super) counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryDirectYieldDenied {
    pub const fn kind(&self) -> WorthQueryDirectYieldDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryYieldTransitionCounters {
        self.counters
    }

    pub fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        &self.paused.active.running.counters
    }

    pub fn into_paused(self) -> WorthQueryPausedDirectGraphExecution {
        self.paused
    }
}

#[must_use = "yield outcomes must be resolved into yielded, denied, or recovery authority"]
pub enum WorthQueryDirectYieldOutcome {
    Yielded(WorthQueryYieldedDirectRun),
    Denied(WorthQueryDirectYieldDenied),
    RecoveryRequired(super::WorthQueryDirectYieldRecoveryRequired),
}

#[must_use = "yielded direct run retains exact cleanup or same-runtime readmission authority"]
pub struct WorthQueryYieldedDirectRun {
    affinity: WorthQueryDirectRunAffinity,
    relational_basis: WorthQueryManagedRelationalObservation,
    bridge: BridgeYieldedExecutionBasis,
    execution: WorthQueryRetainedManagedGraphExecution,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
    inspection: super::WorthQueryYieldedDirectRunInspection,
}

impl WorthQueryYieldedDirectRun {
    pub(super) fn request_for_phase<'a>(
        &self,
        phase: &'a crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        worth_execution::ExecutionRequest<'a, 'a>,
        crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    > {
        phase.request_for_managed(&self.relational_basis)
    }

    pub(in crate::domain_computation::managed_run) fn preflight_retained_provider_call(
        &self,
    ) -> Result<
        crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    >{
        self.affinity.preflight_readmission_call(&self.execution)
    }

    pub(in crate::domain_computation::managed_run) fn query_readmission_denial(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> Option<(super::WorthQueryDirectReadmissionDenialKind, &'static str)> {
        if !self.affinity.belongs_to_runtime(runtime) {
            return Some((
                super::WorthQueryDirectReadmissionDenialKind::ForeignQueryRuntime,
                "yielded run belongs to a different Query execution runtime",
            ));
        }
        if !self.affinity.belongs_to_current_installation(runtime) {
            return Some((
                super::WorthQueryDirectReadmissionDenialKind::StaleInstallationGeneration,
                "yielded run belongs to a stale installed-operation generation",
            ));
        }
        if self.affinity.retained_capacity_reservation_count() == 0 {
            return Some((
                super::WorthQueryDirectReadmissionDenialKind::RetainedCapacityMismatch,
                "yielded run no longer owns its nonempty capacity-reservation package",
            ));
        }
        if !self.relational_basis.is_live() {
            return Some((
                super::WorthQueryDirectReadmissionDenialKind::RelationalLeaseNotLive,
                "yielded Relational execution-basis lease is no longer live",
            ));
        }
        if !self.execution.provider_generation_matches_anchor() {
            return Some((
                super::WorthQueryDirectReadmissionDenialKind::ProviderCheckpointMismatch,
                "provider checkpoint generation no longer matches its retained provider anchor",
            ));
        }
        None
    }

    pub(super) fn owner_from_yield_transition(
        minted: super::direct_yield_transition::WorthQueryDirectYieldMintedOwner,
        _owner: super::direct_yield_transition::WorthQueryDirectYieldMint,
    ) -> Self {
        let inspection = super::WorthQueryYieldedDirectRunInspection::capture(
            &minted.affinity,
            &minted.execution,
            &minted.run_counters,
            minted.yield_counters,
        );
        Self {
            affinity: minted.affinity,
            relational_basis: minted.relational_basis,
            bridge: minted.bridge,
            execution: minted.execution,
            run_counters: minted.run_counters,
            yield_counters: minted.yield_counters,
            inspection,
        }
    }

    pub(in crate::domain_computation::managed_run) fn owner_into_readmission_parts(
        self,
        _owner: &super::WorthQueryDirectReadmissionTransitionPermit,
    ) -> (
        WorthQueryDirectRunAffinity,
        WorthQueryManagedRelationalObservation,
        BridgeYieldedExecutionBasis,
        WorthQueryRetainedManagedGraphExecution,
        WorthQueryManagedRunCounters,
        WorthQueryYieldTransitionCounters,
        super::WorthQueryYieldedDirectRunInspection,
    ) {
        (
            self.affinity,
            self.relational_basis,
            self.bridge,
            self.execution,
            self.run_counters,
            self.yield_counters,
            self.inspection,
        )
    }

    pub(in crate::domain_computation::managed_run) fn owner_restore_from_readmission(
        restored: super::readmission::WorthQueryDirectYieldRestoredOwner,
        _owner: &super::WorthQueryDirectReadmissionTransitionPermit,
    ) -> Self {
        Self {
            affinity: restored.affinity,
            relational_basis: restored.relational_basis,
            bridge: restored.bridge,
            execution: restored.execution,
            run_counters: restored.run_counters,
            yield_counters: restored.yield_counters,
            inspection: restored.inspection,
        }
    }

    pub(super) fn owner_into_cleanup_parts(
        self,
        _owner: &super::direct_yield_cleanup::WorthQueryDirectYieldCleanupPermit,
    ) -> (
        WorthQueryDirectRunAffinity,
        WorthQueryManagedRelationalObservation,
        BridgeYieldedExecutionBasis,
        WorthQueryRetainedManagedGraphExecution,
        WorthQueryManagedRunCounters,
        WorthQueryYieldTransitionCounters,
    ) {
        (
            self.affinity,
            self.relational_basis,
            self.bridge,
            self.execution,
            self.run_counters,
            self.yield_counters,
        )
    }

    pub const fn inspection(&self) -> &super::WorthQueryYieldedDirectRunInspection {
        &self.inspection
    }

    #[must_use = "cleanup returns a direct yielded-run cleanup outcome that must be resolved"]
    pub fn cleanup(self) -> super::WorthQueryDirectYieldCleanupOutcome {
        super::direct_yield_cleanup::cleanup_yielded_direct(self)
    }

    pub fn readmit_same_runtime(
        self,
        active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        query_runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> super::WorthQueryDirectReadmissionOutcome {
        super::readmission::readmit_direct(active_request, self, query_runtime, bridge_runtime)
    }
}
