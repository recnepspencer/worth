use super::*;

impl WorthQueryDirectRunReadmissionPending {
    pub(in crate::domain_computation::managed_run) fn bridge_readmission_intent(
        &self,
        owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> worth_runtime_bridge::facade::BridgeManagedExecutionIntent {
        worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            self.attempt
                .yielded_binding_authority(owner)
                .binding_identity()
                .to_owned(),
            self.attempt.attempt_identity(owner).as_str().to_owned(),
        )
    }

    pub(in crate::domain_computation::managed_run) fn commit(
        self,
        owner: crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectRunAffinity {
        let binding = self.attempt.provider_work_rebinding(&owner);
        let attempt = self.attempt.commit(owner);
        let provider_work = self
            .provider_work
            .rebind_direct_provider_session(binding)
            .unwrap_or_else(|_| {
                panic!("direct provider-work ledger lost its readmission affinity")
            });
        WorthQueryDirectRunAffinity {
            logical: self.logical,
            provider_work,
            attempt,
        }
    }

    pub(in crate::domain_computation::managed_run) fn abort(
        self,
        owner: crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectRunAffinity {
        WorthQueryDirectRunAffinity {
            logical: self.logical,
            attempt: self.attempt.abort(owner),
            provider_work: self.provider_work,
        }
    }

    pub(in crate::domain_computation::managed_run) fn restore_provider(
        mut self,
        active_request: worth_execution::ExecutionRequest<'_, '_>,

        execution: crate::domain_computation::managed_run::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
        contract: crate::domain_computation::managed_run::step_contract_admission::WorthQueryAdmittedManagedStepContract,
        _owner: &crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit,
    ) -> WorthQueryDirectRunProviderRestoreOutcome {
        let fresh_call = self
            .fresh_call
            .take()
            .expect("direct readmission may consume its provider call exactly once");
        match crate::domain_computation::managed_run::provider_restore::restore(active_request,
            execution,
            fresh_call,
            contract,
        ) {
            crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreOutcome::Pending(provider) => {
                WorthQueryDirectRunProviderRestoreOutcome::Pending {
                    resource: self,
                    provider,
                }
            }
            crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreOutcome::Denied(denial) => {
                WorthQueryDirectRunProviderRestoreOutcome::Denied {
                    resource: self,
                    denial,
                }
            }
            crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreOutcome::RecoveryRequired(recovery) => {
                WorthQueryDirectRunProviderRestoreOutcome::RecoveryRequired {
                    resource: self,
                    recovery,
                }
            }
        }
    }
}
