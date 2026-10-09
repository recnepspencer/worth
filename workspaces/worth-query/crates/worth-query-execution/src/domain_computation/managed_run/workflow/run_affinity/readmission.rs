use super::*;

impl WorthQueryWorkflowRunReadmissionPending {
    pub(in crate::domain_computation::managed_run) fn bridge_readmission_intent(
        &self,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> worth_runtime_bridge::facade::BridgeManagedExecutionIntent {
        let owner = WorthQueryWorkflowRunTransitionPermit::mint();
        worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            self.attempt
                .yielded_binding_authority(&owner)
                .binding_identity()
                .to_owned(),
            self.attempt.attempt_identity(&owner).as_str().to_owned(),
        )
    }

    pub(in crate::domain_computation::managed_run) fn restore_provider(
        self,
        active_request: worth_execution::ExecutionRequest<'_, '_>,

        execution: super::super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
        contract: super::super::super::step_contract_admission::WorthQueryAdmittedManagedStepContract,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowRunProviderRestoreOutcome {
        let owner = WorthQueryWorkflowRunTransitionPermit::mint();
        let (attempt, call) = self.attempt.extract_provider_call(&owner);
        let affinity = WorthQueryWorkflowRunRestoredPending {
            logical: self.logical,
            attempt,
            provider_work: self.provider_work,
        };
        match super::super::super::provider_restore::restore(active_request, execution, call, contract) {
            super::super::super::provider_restore::WorthQueryManagedGraphRestoreOutcome::Pending(provider) => {
                WorthQueryWorkflowRunProviderRestoreOutcome::Pending { affinity, provider }
            }
            super::super::super::provider_restore::WorthQueryManagedGraphRestoreOutcome::Denied(denial) => {
                WorthQueryWorkflowRunProviderRestoreOutcome::Denied { affinity, denial }
            }
            super::super::super::provider_restore::WorthQueryManagedGraphRestoreOutcome::RecoveryRequired(recovery) => {
                WorthQueryWorkflowRunProviderRestoreOutcome::RecoveryRequired { affinity, recovery }
            }
        }
    }

    pub(in crate::domain_computation::managed_run) fn abort_progression(
        self,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowRunAffinity {
        WorthQueryWorkflowRunAffinity {
            logical: self.logical,
            attempt: self
                .attempt
                .abort(WorthQueryWorkflowRunTransitionPermit::mint()),
            provider_work: self.provider_work,
        }
    }
}

impl WorthQueryWorkflowRunRestoredPending {
    fn commit(self) -> WorthQueryWorkflowRunAffinity {
        let owner = WorthQueryWorkflowRunTransitionPermit::mint();
        let rebinding = self.attempt.provider_work_rebinding(&owner);
        let attempt = self.attempt.commit(owner);
        let provider_work = self
            .provider_work
            .rebind_workflow_provider_session(rebinding)
            .unwrap_or_else(|_| {
                panic!("workflow provider-work ledger lost its readmission affinity")
            });
        WorthQueryWorkflowRunAffinity {
            logical: self.logical,
            attempt,
            provider_work,
        }
    }

    pub(in crate::domain_computation::managed_run) fn commit_running(
        self,
        state: super::super::super::readmission::WorthQueryWorkflowReadmissionCommitState,
        bridge: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionPending,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> (
        super::super::WorthQueryRunningWorkflowRun,
        worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    ) {
        let (bridge_basis, bridge_counters) = bridge_runtime
            .commit_yielded_execution_basis_readmission(bridge)
            .into_parts();
        let running = state.owner_install(
            self.commit(),
            bridge_basis,
            &WorthQueryWorkflowRunTransitionPermit::mint(),
        );
        (running, bridge_counters)
    }

    pub(in crate::domain_computation::managed_run) fn abort_progression(
        self,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowRunAffinity {
        self.abort()
    }

    pub(in crate::domain_computation::managed_run) fn abort_recovery(
        self,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> WorthQueryWorkflowRunAffinity {
        self.abort()
    }

    fn abort(self) -> WorthQueryWorkflowRunAffinity {
        WorthQueryWorkflowRunAffinity {
            logical: self.logical,
            attempt: self
                .attempt
                .abort(WorthQueryWorkflowRunTransitionPermit::mint()),
            provider_work: self.provider_work,
        }
    }
}
