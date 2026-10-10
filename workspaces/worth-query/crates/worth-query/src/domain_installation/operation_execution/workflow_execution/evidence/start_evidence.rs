use super::WorthQueryWorkflowRunCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowStartDenialKind {
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    RuntimeAuthority(crate::domain_installation::WorthQueryDomainHandleDenialKind),
    WorkflowNotDeclared,
    StageExecutorMissing,
    ArtifactAuthority(crate::domain_installation::WorthQueryArtifactDenial),
    ConditionalExecution(crate::domain_installation::WorthQueryConditionalExecutionDenialKind),
    ConditionalReentry(crate::domain_installation::WorthQueryConditionalAdmissionDenial),
    ManagedRun(String),
}

#[derive(Debug)]
pub struct WorthQueryWorkflowStartDenial {
    kind: WorthQueryWorkflowStartDenialKind,
    counters: WorthQueryWorkflowRunCounters,
}

impl WorthQueryWorkflowStartDenial {
    pub(super) const fn new(
        kind: WorthQueryWorkflowStartDenialKind,
        counters: WorthQueryWorkflowRunCounters,
    ) -> Self {
        Self { kind, counters }
    }

    pub const fn kind(&self) -> &WorthQueryWorkflowStartDenialKind {
        &self.kind
    }

    pub const fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.counters
    }
}
