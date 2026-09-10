use super::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryWorkflowEffectEvidence,
    WorthQueryWorkflowRunCounters, WorthQueryWorkflowStageReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAdvanceDenialKind {
    RuntimeAuthority(crate::domain_installation::WorthQueryDomainHandleDenialKind),
    UnknownStage,
    StageAlreadyCompleted,
    PredecessorIncomplete(String),
    PredecessorAuthorityMissing(String),
    RequiredCapability(String),
    RequiredDomain(String),
    InputContract,
    ArtifactCarriage(crate::domain_installation::WorthQueryArtifactDenial),
    ResourceAdmissionMissing,
    GraphProvider(String),
    StageExecutor {
        class: worth_query_installation::facade::WorthQueryOperationFailureClass,
        detail: String,
    },
    UndeclaredFailureClass(worth_query_installation::facade::WorthQueryOperationFailureClass),
    PrimaryReadEvidence,
    EffectEvidence,
    InvariantEvidence,
    LineageEvidence,
    CostContract,
    OutputContract,
    TerminalContract,
    DomainEvidence(super::WorthQueryDomainEvidenceAdmissionDenialKind),
    ParallelFrontierShape,
    NonDeterministicLowering,
    ParallelProvider(String),
    ParallelNotAdmitted(worth_signal::facade::adapters::FrontierRouteSerialFallbackReason),
    ConditionalExecution(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    ConditionalReentry(crate::domain_installation::WorthQueryConditionalAdmissionDenial),
}

#[derive(Debug)]
pub struct WorthQueryWorkflowAdvanceDenial {
    kind: WorthQueryWorkflowAdvanceDenialKind,
    evidence: Box<WorthQueryWorkflowAdvanceDenialEvidence>,
}

#[derive(Debug)]
struct WorthQueryWorkflowAdvanceDenialEvidence {
    counters: WorthQueryWorkflowRunCounters,
    executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    completed_stage_receipts: Vec<WorthQueryWorkflowStageReceipt>,
    managed_cleanup:
        Option<worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome>,
}

impl WorthQueryWorkflowAdvanceDenial {
    pub(super) fn new(
        kind: WorthQueryWorkflowAdvanceDenialKind,
        counters: WorthQueryWorkflowRunCounters,
    ) -> Self {
        Self {
            kind,
            evidence: Box::new(WorthQueryWorkflowAdvanceDenialEvidence {
                counters,
                executed_effects: Vec::new(),
                graph_receipts: Vec::new(),
                completed_stage_receipts: Vec::new(),
                managed_cleanup: None,
            }),
        }
    }

    pub(super) fn with_executed_effects(
        kind: WorthQueryWorkflowAdvanceDenialKind,
        counters: WorthQueryWorkflowRunCounters,
        executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    ) -> Self {
        Self {
            kind,
            evidence: Box::new(WorthQueryWorkflowAdvanceDenialEvidence {
                counters,
                executed_effects,
                graph_receipts: Vec::new(),
                completed_stage_receipts: Vec::new(),
                managed_cleanup: None,
            }),
        }
    }

    pub fn kind(&self) -> &WorthQueryWorkflowAdvanceDenialKind {
        &self.kind
    }

    pub fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.evidence.counters
    }

    pub fn executed_effects(&self) -> &[WorthQueryWorkflowEffectEvidence] {
        &self.evidence.executed_effects
    }

    pub fn graph_receipts(&self) -> &[WorthQueryBoundGraphExecutionReceipt] {
        &self.evidence.graph_receipts
    }

    pub fn completed_stage_receipts(&self) -> &[WorthQueryWorkflowStageReceipt] {
        &self.evidence.completed_stage_receipts
    }

    pub fn take_managed_cleanup(
        &mut self,
    ) -> Option<worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome> {
        self.evidence.managed_cleanup.take()
    }

    pub(super) fn has_managed_cleanup(&self) -> bool {
        self.evidence.managed_cleanup.is_some()
    }

    pub(super) fn with_managed_cleanup(
        mut self,
        cleanup: worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome,
    ) -> Self {
        self.evidence.managed_cleanup = Some(cleanup);
        self
    }

    pub(super) fn with_graph_receipts(
        mut self,
        graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    ) -> Self {
        self.evidence.graph_receipts = graph_receipts;
        self
    }

    pub(super) fn prepend_executed_effects(
        mut self,
        mut prior_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    ) -> Self {
        prior_effects.append(&mut self.evidence.executed_effects);
        self.evidence.executed_effects = prior_effects;
        self
    }

    pub(super) fn with_completed_stage_receipts(
        mut self,
        receipts: Vec<WorthQueryWorkflowStageReceipt>,
    ) -> Self {
        self.evidence.completed_stage_receipts = receipts;
        self
    }
}
