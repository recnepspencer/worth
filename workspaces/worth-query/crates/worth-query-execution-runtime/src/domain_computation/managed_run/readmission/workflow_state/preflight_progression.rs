use super::*;

impl WorthQueryWorkflowYieldedAssociation {
    pub fn owner_preflight(
        self,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowBridgePreflightTransition {
        match bridge_runtime.preflight_yielded_execution_basis(
            self.bridge,
            self.affinity.binding_identity_projection(),
        ) {
            Ok(bridge) => WorthQueryWorkflowBridgePreflightOutcome::Admitted {
                association: WorthQueryWorkflowPreflightAssociation {
                    state: self.state,
                    affinity: self.affinity,
                    bridge,
                    execution: self.execution,
                },
            },
            Err(denial) => {
                let detail = Arc::from(denial.detail());
                let (bridge, counters) = denial.into_returned_yielded().into_parts();
                WorthQueryWorkflowBridgePreflightOutcome::Denied {
                    yielded: self
                        .state
                        .restore_yielded(self.affinity, bridge, self.execution),
                    detail,
                    counters,
                }
            }
        }
        .into()
    }
}

pub struct WorthQueryWorkflowBridgePreflightTransition {
    outcome: WorthQueryWorkflowBridgePreflightOutcome,
}

enum WorthQueryWorkflowBridgePreflightOutcome {
    Admitted {
        association: WorthQueryWorkflowPreflightAssociation,
    },
    Denied {
        yielded: WorthQueryYieldedWorkflowRun,
        detail: Arc<str>,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
}

impl From<WorthQueryWorkflowBridgePreflightOutcome>
    for WorthQueryWorkflowBridgePreflightTransition
{
    fn from(outcome: WorthQueryWorkflowBridgePreflightOutcome) -> Self {
        Self { outcome }
    }
}

impl WorthQueryWorkflowBridgePreflightTransition {
    pub(in super::super) fn owner_resolve(
        self,
    ) -> Result<
        WorthQueryWorkflowPreflightAssociation,
        super::super::workflow_preflight::WorthQueryWorkflowResumePreflightDenied,
    > {
        match self.outcome {
            WorthQueryWorkflowBridgePreflightOutcome::Admitted { association } => Ok(association),
            WorthQueryWorkflowBridgePreflightOutcome::Denied {
                yielded,
                detail,
                counters,
            } => Err(super::super::workflow_preflight::WorthQueryWorkflowResumePreflightDenied::new(
                crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenialKind::BridgeReadmissionDenied,
                detail,
                yielded,
                Some(counters),
            )),
        }
    }
}

impl WorthQueryWorkflowPreflightAssociation {
    pub fn execution_contract(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstalledBoundedStepContract {
        self.execution.contract().clone()
    }

    pub fn step_contract(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeManagedExecutionStepContract {
        self.bridge.step_contract()
    }

    pub fn owner_abort(
        self,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> (
        WorthQueryYieldedWorkflowRun,
        worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    ) {
        let (bridge, counters) = self.bridge.into_returned_yielded().into_parts();
        (
            self.state
                .restore_yielded(self.affinity, bridge, self.execution),
            counters,
        )
    }

    pub fn owner_begin_resource(
        self,
        resources: Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        call: crate::domain_computation::provider_session::graph_provider::WorthQueryGraphProviderCallReadmissionPlan,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowProvisionalAssociation {
        WorthQueryWorkflowProvisionalAssociation {
            state: self.state,
            resource: self.affinity.begin_readmission(resources, call, _owner),
            bridge: self.bridge,
            execution: self.execution,
        }
    }
}

impl WorthQueryWorkflowProvisionalAssociation {
    pub fn owner_readmit_bridge(
        self,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowBridgeReadmissionTransition {
        let intent = self.resource.bridge_readmission_intent(_owner);
        let outcome = match bridge_runtime.readmit_yielded_execution_basis(self.bridge, intent) {
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionOutcome::Pending(
                bridge,
            ) => WorthQueryWorkflowBridgeReadmissionOutcome::Pending {
                counters: bridge.counters(),
                association: WorthQueryWorkflowBridgePendingAssociation {
                    state: self.state,
                    resource: self.resource,
                    bridge,
                    execution: self.execution,
                },
            },
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionOutcome::Denied(
                denial,
            ) => {
                let detail = Arc::from(denial.detail());
                let (bridge, counters) = denial.into_returned_yielded().into_parts();
                WorthQueryWorkflowBridgeReadmissionOutcome::Denied {
                    detail,
                    counters,
                    yielded: self.state.restore_yielded(
                        self.resource.abort_progression(_owner),
                        bridge,
                        self.execution,
                    ),
                }
            }
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => WorthQueryWorkflowBridgeReadmissionOutcome::RecoveryRequired {
                detail: Arc::from(recovery.detail()),
                counters: recovery.counters(),
                association: WorthQueryWorkflowBridgeRecoveryAssociation {
                    state: self.state,
                    affinity: self.resource.abort_progression(_owner),
                    bridge: recovery,
                    execution: self.execution,
                },
            },
        };
        WorthQueryWorkflowBridgeReadmissionTransition { outcome }
    }
}

pub struct WorthQueryWorkflowBridgeReadmissionTransition {
    outcome: WorthQueryWorkflowBridgeReadmissionOutcome,
}

enum WorthQueryWorkflowBridgeReadmissionOutcome {
    Pending {
        association: WorthQueryWorkflowBridgePendingAssociation,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
    Denied {
        yielded: WorthQueryYieldedWorkflowRun,
        detail: Arc<str>,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
    RecoveryRequired {
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        detail: Arc<str>,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
}

impl WorthQueryWorkflowBridgeReadmissionTransition {
    pub fn owner_resolve(
        self,
        contract: crate::domain_computation::managed_run::step_contract_admission::WorthQueryAdmittedManagedStepContract,
        stage_identity: String,
        mut progress: crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Result<super::super::WorthQueryWorkflowBridgeReadmissionPending, crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome>{
        match self.outcome {
            WorthQueryWorkflowBridgeReadmissionOutcome::Pending {
                association,
                counters,
            } => {
                progress.observe_bridge(counters);
                Ok(super::super::WorthQueryWorkflowBridgeReadmissionPending {
                    association,
                    contract,
                    stage_identity,
                    progress,
                })
            }
            WorthQueryWorkflowBridgeReadmissionOutcome::Denied {
                yielded,
                detail,
                counters,
            } => {
                progress.observe_bridge(counters);
                Err(super::super::denied(
                    crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenialKind::BridgeReadmissionDenied,
                    detail,
                    yielded,
                    progress,
                ))
            }
            WorthQueryWorkflowBridgeReadmissionOutcome::RecoveryRequired {
                association,
                detail,
                counters,
            } => {
                progress.observe_bridge(counters);
                Err(crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                    crate::domain_computation::managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryRequired::bridge_cleanup(
                        detail,
                        progress,
                        association,
                        owner,
                    ),
                ))
            }
        }
    }
}
