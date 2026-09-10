use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::domain_computation::provider_session::graph_provider::{
    WorthQueryGraphProviderCall, WorthQueryGraphProviderCallReadmissionPlan,
};
use crate::domain_computation::provider_session::WorthQueryDirectResourceReadmissionPending;
use crate::domain_computation::WorthQueryDirectExecutionResourceAttempt;

use super::provider_work::{
    WorthQueryManagedProviderCleanupAuthority, WorthQueryManagedProviderWorkEvidence,
    WorthQueryManagedProviderWorkLedger,
};

mod readmission;
mod terminal;

pub(super) use terminal::WorthQueryDirectRunTerminalAffinity;

static NEXT_MANAGED_LOGICAL_RUN: AtomicU64 = AtomicU64::new(1);

/// Move-only association between one logical direct run and its current live
/// resource attempt. The current attempt identity and provider session are
/// always read from the owned attempt; they cannot be substituted separately.
pub(super) struct WorthQueryDirectRunAffinity {
    logical: Arc<str>,
    attempt: WorthQueryDirectExecutionResourceAttempt,
    provider_work: WorthQueryManagedProviderWorkLedger,
}

pub(super) struct WorthQueryDirectRunReadmissionPending {
    logical: Arc<str>,
    attempt: WorthQueryDirectResourceReadmissionPending,
    provider_work: WorthQueryManagedProviderWorkLedger,
    fresh_call: Option<WorthQueryGraphProviderCall>,
}

pub(super) enum WorthQueryDirectRunProviderRestoreOutcome {
    Pending {
        resource: WorthQueryDirectRunReadmissionPending,
        provider: super::provider_restore::WorthQueryManagedGraphRestorePending,
    },
    Denied {
        resource: WorthQueryDirectRunReadmissionPending,
        denial: super::provider_restore::WorthQueryManagedGraphRestoreDenied,
    },
    RecoveryRequired {
        resource: WorthQueryDirectRunReadmissionPending,
        recovery: super::provider_restore::WorthQueryManagedGraphRestoreRecoveryRequired,
    },
}

impl WorthQueryDirectRunAffinity {
    pub(super) fn initial(attempt: WorthQueryDirectExecutionResourceAttempt) -> Self {
        let ordinal = next_managed_logical_run_ordinal(&NEXT_MANAGED_LOGICAL_RUN)
            .expect("managed logical-run identity space must not be exhausted");
        let provider_work = WorthQueryManagedProviderWorkLedger::new(
            attempt.managed_provider_session().closed_identity(),
        );
        Self {
            logical: Arc::from(format!("managed-logical-run:{ordinal}")),
            attempt,
            provider_work,
        }
    }

    pub(super) fn logical_identity(&self) -> &str {
        &self.logical
    }

    pub(super) fn attempt_identity(&self) -> &str {
        self.attempt.attempt_identity().as_str()
    }

    pub(super) fn belongs_to_operation(
        &self,
        operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        self.attempt.binding_authority().binding_identity() == operation.binding_identity()
    }

    pub(super) fn graph_work_affinity(
        &self,
    ) -> Option<crate::domain_computation::operation_binding::WorthQueryApplicationGraphWorkAffinity>
    {
        self.attempt.binding_authority().graph_work_affinity()
    }

    pub(super) fn resource_release_expectation(&self) -> (&str, usize) {
        (
            self.attempt.resources().identity(),
            self.attempt.retained_capacity_reservation_count(),
        )
    }

    pub(super) fn evidence(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionResourceAttemptEvidence {
        self.attempt.evidence()
    }

    pub(super) fn observe_safe_point(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeBoundExecutionBasis,
    ) -> Result<
        super::WorthQueryManagedSafePointObservation,
        super::WorthQueryManagedSafePointFailure,
    > {
        super::safe_point_observation::observe_managed_run_safe_point(
            self.attempt.attempt_identity().description_arc(),
            bridge,
        )
    }

    pub(super) fn provider_work_mut(&mut self) -> &mut WorthQueryManagedProviderWorkLedger {
        &mut self.provider_work
    }

    pub(super) fn provider_work_has_uncertainty(&self) -> bool {
        self.provider_work.has_uncertainty()
    }

    pub(super) fn provider_work_snapshot(&self) -> WorthQueryManagedProviderWorkEvidence {
        self.provider_work.snapshot()
    }

    pub(super) fn provider_session_description(&self) -> &str {
        self.attempt.managed_provider_session().identity()
    }

    pub(super) fn retained_capacity_reservation_count(&self) -> usize {
        self.attempt.retained_capacity_reservation_count()
    }

    pub(super) fn operation_binding_identity(&self) -> &str {
        self.attempt.binding_authority().binding_identity()
    }

    pub(super) fn installed_operation_identity(&self) -> &str {
        self.attempt.binding_authority().operation_identity()
    }

    pub(super) fn semantic_basis_identity(&self) -> &str {
        self.attempt.binding_authority().basis_identity()
    }

    pub(super) fn installation_generation(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstallationGeneration {
        self.attempt.binding_authority().installation_generation()
    }

    pub(super) fn installation_is_current(&self) -> bool {
        self.attempt
            .binding_authority()
            .is_current_installation_generation()
    }

    pub(super) fn yield_is_installed(&self) -> bool {
        self.attempt
            .resources()
            .envelope()
            .yield_contract()
            .is_some()
    }

    pub(super) fn yield_retained_bytes_ceiling(&self) -> Option<u64> {
        self.attempt
            .resources()
            .envelope()
            .yield_contract()
            .map(|contract| contract.retained_bytes_ceiling())
    }

    pub(super) fn provider_plan_operation(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.attempt.binding_authority()
    }

    pub(super) fn provider_plan_session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.attempt.managed_provider_session()
    }

    pub(super) fn provider_plan_resources(
        &self,
    ) -> (
        &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan,
        &crate::domain_computation::WorthQueryExecutionResourceAttemptEvidence,
    ) {
        (self.attempt.resources(), self.attempt.evidence())
    }

    pub(super) fn graph_resource_support(
        &self,
        role: &str,
    ) -> Option<
        &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    > {
        self.attempt
            .resources()
            .support_snapshot()
            .graph_provider(role)
    }

    pub(super) fn bind_graph_provider_call(
        &self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: crate::domain_computation::WorthQueryGraphProviderCallRequest,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphProviderCall,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        self.attempt
            .managed_provider_session()
            .bind_graph_provider_call(
                graph_authority,
                request,
                self.attempt.evidence(),
                self.attempt.resources().shared_envelope(),
            )
    }

    pub(super) fn preflight_readmission_call(
        &self,
        execution: &super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
    ) -> Result<
        WorthQueryGraphProviderCallReadmissionPlan,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        execution
            .call
            .preflight_readmission(self.attempt.binding_authority(), self.attempt.evidence())
    }

    pub(super) fn belongs_to_runtime(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt.binding_authority().belongs_to(runtime)
    }

    pub(super) fn belongs_to_current_installation(
        &self,
        runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> bool {
        self.attempt
            .binding_authority()
            .belongs_to_current_installation(runtime)
    }

    pub(super) fn begin_readmission(
        self,
        call: WorthQueryGraphProviderCallReadmissionPlan,
        owner: &super::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectRunReadmissionPending {
        let (attempt, fresh_call) =
            WorthQueryDirectResourceReadmissionPending::begin(self.attempt, call, owner);
        WorthQueryDirectRunReadmissionPending {
            logical: self.logical,
            attempt,
            provider_work: self.provider_work,
            fresh_call: Some(fresh_call),
        }
    }
}

impl WorthQueryDirectRunAffinity {
    pub(super) fn into_terminal_parts(
        self,
    ) -> (
        WorthQueryDirectRunTerminalAffinity,
        WorthQueryManagedProviderWorkEvidence,
        WorthQueryManagedProviderCleanupAuthority,
    ) {
        let (provider_work, provider_cleanup) = self.provider_work.into_terminal_parts();
        (
            WorthQueryDirectRunTerminalAffinity::new(self.logical, self.attempt),
            provider_work,
            provider_cleanup,
        )
    }
}

fn next_managed_logical_run_ordinal(counter: &AtomicU64) -> Option<u64> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_logical_run_ordinal_exhaustion_cannot_wrap() {
        let counter = AtomicU64::new(u64::MAX - 1);

        assert_eq!(
            next_managed_logical_run_ordinal(&counter),
            Some(u64::MAX - 1)
        );
        assert_eq!(next_managed_logical_run_ordinal(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
