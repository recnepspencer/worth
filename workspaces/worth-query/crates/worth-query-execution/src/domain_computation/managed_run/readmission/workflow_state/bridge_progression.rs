use super::*;

impl WorthQueryWorkflowBridgePendingAssociation {
    pub(in crate::domain_computation::managed_run::readmission) fn owner_restore_provider(
        self,
        active_request: worth_execution::ExecutionRequest<'_, '_>,

        contract: crate::domain_computation::managed_run::step_contract_admission::WorthQueryAdmittedManagedStepContract,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowProviderRestoreTransition {
        let outcome = match self.resource.restore_provider(active_request, self.execution, contract, _owner) {
            crate::domain_computation::managed_run::workflow::WorthQueryWorkflowRunProviderRestoreOutcome::Pending {
                affinity,
                provider,
            } => WorthQueryWorkflowProviderRestoreOutcome::Pending {
                association: WorthQueryWorkflowRestoredAssociation {
                    state: self.state,
                    resource: affinity,
                    bridge: self.bridge,
                },
                provider,
            },
            crate::domain_computation::managed_run::workflow::WorthQueryWorkflowRunProviderRestoreOutcome::Denied {
                affinity,
                denial,
            } => WorthQueryWorkflowProviderRestoreOutcome::Denied {
                association: WorthQueryWorkflowRestoredAssociation {
                    state: self.state,
                    resource: affinity,
                    bridge: self.bridge,
                },
                denial,
            },
            crate::domain_computation::managed_run::workflow::WorthQueryWorkflowRunProviderRestoreOutcome::RecoveryRequired {
                affinity,
                recovery,
            } => WorthQueryWorkflowProviderRestoreOutcome::RecoveryRequired {
                association: WorthQueryWorkflowRestoredAssociation {
                    state: self.state,
                    resource: affinity,
                    bridge: self.bridge,
                },
                recovery,
            },
        };
        WorthQueryWorkflowProviderRestoreTransition { outcome }
    }
}

impl WorthQueryWorkflowRestoredAssociation {
    pub(in crate::domain_computation::managed_run::readmission) fn owner_prepare_artifact_generation(
        &self,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Result<
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionGenerationPending,
        crate::domain_computation::WorthQueryArtifactDenial,
    > {
        self.state.artifacts.registry().prepare_next_generation()
    }

    pub(in crate::domain_computation::managed_run::readmission) fn owner_artifact_context(
        &self,
        stage_identity: &str,
        generation: &crate::domain_computation::artifact_owner::WorthQueryArtifactProductionGenerationPending,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Result<
        Option<
            crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext,
        >,
        crate::domain_computation::WorthQueryArtifactDenial,
    >{
        self.state
            .artifacts
            .production_authority_for_readmission(stage_identity, generation)
            .map(|authority| {
                authority.map(|authority| {
                    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext::new(
                        authority,
                        Arc::clone(&self.state.provider_artifact_occurrences),
                    )
                })
            })
    }

    pub(in crate::domain_computation::managed_run::readmission) fn owner_commit_generation(
        self,
        committed: WorthQueryArtifactProductionGenerationCommitted,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowCommittedAssociation {
        WorthQueryWorkflowCommittedAssociation::owner_from_restored(self, committed, owner)
    }

    pub(in crate::domain_computation::managed_run::readmission) fn owner_abort_bridge(
        self,
        execution: WorthQueryRetainedManagedGraphExecution,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> WorthQueryWorkflowBridgeAbortTransition {
        match self.bridge.abort() {
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCleanupOutcome::Complete(
                returned,
            ) => {
                let (bridge, counters) = returned.into_parts();
                WorthQueryWorkflowBridgeAbortOutcome::Yielded {
                    yielded: self.state.restore_yielded(
                        self.resource.abort_progression(_owner),
                        bridge,
                        execution,
                    ),
                    counters,
                }
            }
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(
                recovery,
            ) => WorthQueryWorkflowBridgeAbortOutcome::RecoveryRequired {
                counters: recovery.counters(),
                association: WorthQueryWorkflowBridgeRecoveryAssociation {
                    state: self.state,
                    affinity: self.resource.abort_progression(_owner),
                    bridge: recovery,
                    execution,
                },
            },
        }
        .into()
    }
}

pub(in crate::domain_computation::managed_run::readmission) struct WorthQueryWorkflowBridgeAbortTransition
{
    outcome: WorthQueryWorkflowBridgeAbortOutcome,
}

enum WorthQueryWorkflowBridgeAbortOutcome {
    Yielded {
        yielded: WorthQueryYieldedWorkflowRun,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
    RecoveryRequired {
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
}

impl From<WorthQueryWorkflowBridgeAbortOutcome> for WorthQueryWorkflowBridgeAbortTransition {
    fn from(outcome: WorthQueryWorkflowBridgeAbortOutcome) -> Self {
        Self { outcome }
    }
}

impl WorthQueryWorkflowBridgeAbortTransition {
    pub(in crate::domain_computation::managed_run::readmission) fn owner_resolve_denial(
        self,
        kind: crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenialKind,
        detail: Arc<str>,
        mut progress: crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome{
        match self.outcome {
            WorthQueryWorkflowBridgeAbortOutcome::Yielded { yielded, counters } => {
                progress.observe_bridge(counters);
                crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome::Denied(
                    crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenied::new(
                        kind,
                        detail,
                        yielded,
                        progress.evidence(),
                    ),
                )
            }
            WorthQueryWorkflowBridgeAbortOutcome::RecoveryRequired {
                association,
                counters,
            } => {
                progress.observe_bridge(counters);
                crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                    crate::domain_computation::managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryRequired::bridge_cleanup(
                        format!("{detail}; Bridge cleanup failed"),
                        progress,
                        association,
                        owner,
                    ),
                )
            }
        }
    }
}

pub(in crate::domain_computation::managed_run::readmission) struct WorthQueryWorkflowProviderRestoreTransition
{
    outcome: WorthQueryWorkflowProviderRestoreOutcome,
}

enum WorthQueryWorkflowProviderRestoreOutcome {
    Pending {
        association: WorthQueryWorkflowRestoredAssociation,
        provider: crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestorePending,
    },
    Denied {
        association: WorthQueryWorkflowRestoredAssociation,
        denial: crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreDenied,
    },
    RecoveryRequired {
        association: WorthQueryWorkflowRestoredAssociation,
        recovery: crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreRecoveryRequired,
    },
}

impl WorthQueryWorkflowProviderRestoreTransition {
    pub(in crate::domain_computation::managed_run::readmission) fn owner_resolve(
        self,
        stage_identity: String,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
        progress: crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress,
        owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome{
        match self.outcome {
            WorthQueryWorkflowProviderRestoreOutcome::Pending {
                association,
                provider,
            } => super::super::workflow_completion::advance_artifact_generation(
                association,
                stage_identity,
                provider,
                bridge_runtime,
                progress,
                owner,
            ),
            WorthQueryWorkflowProviderRestoreOutcome::Denied {
                association,
                denial,
            } => super::super::workflow_abort::abort_without_provider(
                crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenialKind::ProviderRestoreDenied,
                Arc::from(denial.detail()),
                association,
                denial.into_retained(),
                progress,
                owner,
            ),
            WorthQueryWorkflowProviderRestoreOutcome::RecoveryRequired {
                association,
                recovery,
            } => {
                let kind = super::super::workflow_abort::map_recovery_kind(recovery.kind());
                let detail = Arc::from(recovery.detail());
                crate::domain_computation::managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                    crate::domain_computation::managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryRequired::provider(
                        kind,
                        detail,
                        progress,
                        association,
                        recovery,
                        owner,
                    ),
                )
            }
        }
    }
}
