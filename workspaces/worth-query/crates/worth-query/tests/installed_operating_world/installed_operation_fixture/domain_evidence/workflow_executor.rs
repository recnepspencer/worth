use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use worth_query::facade::{domain, read};

use super::super::{GeometryDomain, ReadFamily, WorkflowRead};
use super::material::{workflow_evidence_material, EvidenceScenario};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EvidenceWorkflowMode {
    Honest = 0,
    OmitSidecars = 1,
    LedgerRegression = 2,
    ReplayCoreDrift = 3,
    OutputOccurrenceMismatch = 4,
}

impl EvidenceWorkflowMode {
    fn scenario(self) -> EvidenceScenario {
        match self {
            Self::Honest => EvidenceScenario::Honest,
            Self::OmitSidecars => EvidenceScenario::OmitSidecars,
            Self::LedgerRegression => EvidenceScenario::LedgerRegression,
            Self::ReplayCoreDrift => EvidenceScenario::ReplayCoreDrift,
            Self::OutputOccurrenceMismatch => EvidenceScenario::OutputOccurrenceMismatch,
        }
    }

    fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Honest,
            1 => Self::OmitSidecars,
            2 => Self::LedgerRegression,
            3 => Self::ReplayCoreDrift,
            4 => Self::OutputOccurrenceMismatch,
            _ => panic!("invalid evidence workflow mode"),
        }
    }
}

#[derive(Clone)]
pub struct EvidenceWorkflowProbe {
    mode: Arc<AtomicU8>,
}

impl EvidenceWorkflowProbe {
    pub fn set(&self, mode: EvidenceWorkflowMode) {
        self.mode.store(mode as u8, Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub(super) struct EvidenceWorkflowExecutor {
    mode: Arc<AtomicU8>,
}

pub(super) struct EvidenceGraphWorkflowExecutor(EvidenceWorkflowExecutor);

impl EvidenceGraphWorkflowExecutor {
    pub(super) fn new() -> (Self, EvidenceWorkflowProbe) {
        let (executor, probe) = EvidenceWorkflowExecutor::new();
        (Self(executor), probe)
    }
}

impl EvidenceWorkflowExecutor {
    pub(super) fn new() -> (Self, EvidenceWorkflowProbe) {
        let mode = Arc::new(AtomicU8::new(EvidenceWorkflowMode::Honest as u8));
        (
            Self {
                mode: Arc::clone(&mode),
            },
            EvidenceWorkflowProbe { mode },
        )
    }

    fn mode(&self) -> EvidenceWorkflowMode {
        EvidenceWorkflowMode::from_raw(self.mode.load(Ordering::SeqCst))
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for EvidenceWorkflowExecutor
{
    const LOWERING_FAMILY: &'static str = "domain-evidence-workflow-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> =
        Some("domain-evidence-workflow-exact-v1");

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::super::executors::installed_read_declaration())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (_input, _computed, context, workspace) = application.into_parts();
        let stage = context.stage().identity();
        if stage == "publish" {
            return Ok(domain::WorthQueryWorkflowStageMaterial::projection(
                "model",
                context.execute_installed_read("model", workspace)?,
            )
            .with_result_state(domain::WorthQueryOperationResultState::Ready));
        }

        let output = domain::WorthQueryWorkflowValue::Text(stage.into());
        let occurrence_identity = output.domain_evidence_occurrence_identity();
        let mut material = domain::WorthQueryWorkflowStageMaterial::new(output);
        if matches!(stage, "start" | "left") {
            material = material.with_domain_evidence(workflow_evidence_material(
                &occurrence_identity,
                self.mode().scenario(),
                stage,
            ));
        }
        Ok(material)
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for EvidenceWorkflowExecutor
{
    fn compare_replay_semantics(
        &self,
        _original: &domain::WorthQueryWorkflowTraceSemantics,
        _replay: &domain::WorthQueryWorkflowTraceSemantics,
        _noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::WorthQueryReplayComparison::Equivalent
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for EvidenceGraphWorkflowExecutor
{
    const LOWERING_FAMILY: &'static str = "domain-evidence-workflow-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> =
        Some("domain-evidence-workflow-exact-v1");

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        self.0.installed_read_declaration()
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.0.execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        self.0.apply(application)
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for EvidenceGraphWorkflowExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        self.0.compare_replay_semantics(original, replay, noise)
    }
}
