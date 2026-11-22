use std::sync::Arc;

use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext;

use super::managed_graph_execution::WorthQueryManagedGraphExecution;
use super::{
    WorthQueryActiveWorkflowGraphExecution, WorthQueryManagedGraphCallRequest,
    WorthQueryRunningWorkflowRun,
};
use crate::domain_computation::WorthQueryGraphProviderCall;

struct WorthQueryBoundWorkflowGraphStart {
    running: WorthQueryRunningWorkflowRun,
    anchor: Arc<WorthQueryGraphProviderAnchor>,
    call: WorthQueryGraphProviderCall,
}

struct WorthQueryReadyWorkflowGraphStart {
    bound: WorthQueryBoundWorkflowGraphStart,
    contract: super::step_contract_admission::WorthQueryAdmittedManagedStepContract,
    artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowGraphExecutionStartFailureKind {
    /// The phase belongs to another installed runtime.
    ForeignAdvancementPhase,
    StageResourcesUnavailable,
    ProviderBinding,
    MissingInstalledProvider,
    ProviderSupportMismatch,
    ProviderStart,
    ProviderStartPanicked,
    ProviderStartContractDenied,
    ProviderStartMemoryLeaked,
    ProviderStartReleaseRecoveryRequired,
    InvalidStepContract,
    StepContract(super::WorthQueryManagedStepContractDenialKind),
    ArtifactAuthority,
}

pub struct WorthQueryWorkflowGraphExecutionStartFailure {
    kind: WorthQueryWorkflowGraphExecutionStartFailureKind,
    detail: Arc<str>,
    running: WorthQueryRunningWorkflowRun,
    provider_retained_bytes: u64,
    provider_retained_allocation_count: u64,
    provider_execution_release:
        Option<crate::domain_computation::WorthQueryProviderExecutionReleaseEvidence>,
}

impl WorthQueryWorkflowGraphExecutionStartFailure {
    pub const fn kind(&self) -> WorthQueryWorkflowGraphExecutionStartFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_running(self) -> WorthQueryRunningWorkflowRun {
        self.running
    }

    pub const fn provider_retained_bytes(&self) -> u64 {
        self.provider_retained_bytes
    }

    pub const fn provider_retained_allocation_count(&self) -> u64 {
        self.provider_retained_allocation_count
    }

    pub const fn provider_execution_release(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryProviderExecutionReleaseEvidence> {
        self.provider_execution_release.as_ref()
    }
}

impl std::fmt::Debug for WorthQueryWorkflowGraphExecutionStartFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowGraphExecutionStartFailure")
            .field("kind", &self.kind)
            .field("detail", &self.detail)
            .field("run_identity", &self.running.identity())
            .finish()
    }
}

pub(super) fn begin(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    running: WorthQueryRunningWorkflowRun,
    stage_identity: &str,
    graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    request: WorthQueryManagedGraphCallRequest,
) -> Result<WorthQueryActiveWorkflowGraphExecution, WorthQueryWorkflowGraphExecutionStartFailure> {
    if execution
        .request_for_managed(running.relational_basis())
        .is_err()
    {
        return Err(start_failure(
            WorthQueryWorkflowGraphExecutionStartFailureKind::ForeignAdvancementPhase,
            "phase belongs to another advancement runtime",
            running,
        ));
    }
    WorthQueryBoundWorkflowGraphStart::bind(running, stage_identity, graph_authority, request)?
        .validate_contract(stage_identity)?
        .start_provider(execution)
}

impl WorthQueryBoundWorkflowGraphStart {
    fn bind(
        running: WorthQueryRunningWorkflowRun,
        stage_identity: &str,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<Self, WorthQueryWorkflowGraphExecutionStartFailure> {
        let Some(anchor) =
            graph_authority.retain_provider_anchor::<WorthQueryGraphProviderAnchor>()
        else {
            return Err(start_failure(
                WorthQueryWorkflowGraphExecutionStartFailureKind::MissingInstalledProvider,
                "installed graph authority does not retain the exact execution-owned provider anchor",
                running,
            ));
        };
        if !running.stage_graph_support_matches(
            stage_identity,
            graph_authority.role(),
            anchor.resource_support(),
        ) {
            return Err(start_failure(
                WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderSupportMismatch,
                "admitted stage graph support was not minted from the installed provider's exact support authority",
                running,
            ));
        }
        let call = match running.mint_stage_graph_provider_call(
            stage_identity,
            graph_authority,
            request,
        ) {
            Ok(call) => call,
            Err("workflow-stage-resources-unavailable") => {
                return Err(start_failure(
                    WorthQueryWorkflowGraphExecutionStartFailureKind::StageResourcesUnavailable,
                    "workflow stage has no admitted graph resources",
                    running,
                ))
            }
            Err(detail) => {
                return Err(start_failure(
                    WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderBinding,
                    detail,
                    running,
                ))
            }
        };
        Ok(Self {
            running,
            anchor,
            call,
        })
    }

    fn validate_contract(
        self,
        stage_identity: &str,
    ) -> Result<WorthQueryReadyWorkflowGraphStart, WorthQueryWorkflowGraphExecutionStartFailure>
    {
        let contract = match self.call.resource_envelope().bounded_step_contract() {
            Ok(contract) => contract,
            Err(detail) => {
                return Err(start_failure(
                    WorthQueryWorkflowGraphExecutionStartFailureKind::InvalidStepContract,
                    detail,
                    self.running,
                ))
            }
        };
        let contract = match self.running.admit_bridge_step_contract(contract) {
            Ok(contract) => contract,
            Err(denial) => return Err(step_contract_failure(denial, self.running)),
        };
        let artifact_context = match self.running.stage_artifact_context(stage_identity) {
            Ok(context) => context,
            Err(denial) => {
                return Err(start_failure(
                    WorthQueryWorkflowGraphExecutionStartFailureKind::ArtifactAuthority,
                    denial.detail(),
                    self.running,
                ))
            }
        };
        Ok(WorthQueryReadyWorkflowGraphStart {
            bound: self,
            contract,
            artifact_context,
        })
    }
}

impl WorthQueryReadyWorkflowGraphStart {
    fn start_provider(
        mut self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryActiveWorkflowGraphExecution, WorthQueryWorkflowGraphExecutionStartFailure>
    {
        self.bound.running.begin_provider_step_call();
        let started = match super::provider_start::start_managed_provider(
            execution
                .request_for_managed(self.bound.running.relational_basis())
                .expect("start admitted this runtime phase"),
            &self.bound.anchor,
            &self.bound.call,
            self.contract.installed().retained_bytes_ceiling(),
        ) {
            Ok(started) => started,
            Err(failure) => {
                let super::provider_start::WorthQueryManagedProviderStartFailure {
                    kind,
                    detail,
                    memory,
                    provider_execution_release,
                } = failure;
                let snapshot = memory.snapshot();
                if let Some(release) = &provider_execution_release {
                    self.bound
                        .running
                        .record_provider_execution_release(release);
                }
                self.bound.running.retain_provider_memory(memory);
                self.bound.running.abandon_provider_step_call();
                return Err(provider_start_failure(
                    kind,
                    detail,
                    snapshot,
                    provider_execution_release,
                    self.bound.running,
                ));
            }
        };
        self.bound
            .running
            .observe_active_provider_memory(started.memory.snapshot());
        Ok(WorthQueryActiveWorkflowGraphExecution::new(
            self.bound.running,
            WorthQueryManagedGraphExecution::new(
                super::managed_graph_execution::WorthQueryManagedGraphExecutionStartParts {
                    call: self.bound.call,
                    execution: started.execution,
                    anchor: self.bound.anchor,
                    contract: self.contract,
                    artifact_context: self.artifact_context,
                    memory: started.memory,
                },
            ),
        ))
    }
}

fn start_failure(
    kind: WorthQueryWorkflowGraphExecutionStartFailureKind,
    detail: impl Into<Arc<str>>,
    running: WorthQueryRunningWorkflowRun,
) -> WorthQueryWorkflowGraphExecutionStartFailure {
    WorthQueryWorkflowGraphExecutionStartFailure {
        kind,
        detail: detail.into(),
        running,
        provider_retained_bytes: 0,
        provider_retained_allocation_count: 0,
        provider_execution_release: None,
    }
}

fn provider_start_failure(
    failure_kind: super::provider_start::WorthQueryManagedProviderStartFailureKind,
    detail: Arc<str>,
    memory: crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemorySnapshot,
    provider_execution_release: Option<
        crate::domain_computation::WorthQueryProviderExecutionReleaseEvidence,
    >,
    running: WorthQueryRunningWorkflowRun,
) -> WorthQueryWorkflowGraphExecutionStartFailure {
    use super::provider_start::WorthQueryManagedProviderStartFailureKind as Kind;
    let kind = match failure_kind {
        Kind::Rejected => WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderStart,
        Kind::Panicked => WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderStartPanicked,
        Kind::ContractDenied => {
            WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderStartContractDenied
        }
        Kind::MemoryLeaked => {
            WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderStartMemoryLeaked
        }
        Kind::ProviderExecutionReleaseRecoveryRequired => {
            WorthQueryWorkflowGraphExecutionStartFailureKind::ProviderStartReleaseRecoveryRequired
        }
    };
    WorthQueryWorkflowGraphExecutionStartFailure {
        kind,
        detail,
        running,
        provider_retained_bytes: memory.retained_bytes(),
        provider_retained_allocation_count: memory.retained_allocation_count(),
        provider_execution_release,
    }
}

fn step_contract_failure(
    denial: super::step_contract_admission::WorthQueryManagedStepContractDenial,
    running: WorthQueryRunningWorkflowRun,
) -> WorthQueryWorkflowGraphExecutionStartFailure {
    WorthQueryWorkflowGraphExecutionStartFailure {
        kind: WorthQueryWorkflowGraphExecutionStartFailureKind::StepContract(denial.kind()),
        detail: Arc::from(denial.detail()),
        running,
        provider_retained_bytes: 0,
        provider_retained_allocation_count: 0,
        provider_execution_release: None,
    }
}
