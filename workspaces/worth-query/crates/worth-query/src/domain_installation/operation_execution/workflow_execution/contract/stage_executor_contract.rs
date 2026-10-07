use super::{
    WorthQueryWorkflowEffectEvidence, WorthQueryWorkflowPrimaryReadEvidence,
    WorthQueryWorkflowStageExecutionContext, WorthQueryWorkflowStageWorkspace,
};

#[path = "stage_application.rs"]
mod application;
#[path = "stage_computation.rs"]
pub(in crate::domain_installation::operation_execution) mod computation;
#[path = "computation_declaration.rs"]
mod computation_declaration;
#[path = "computation_storage.rs"]
mod computation_storage;
#[path = "stage_preparation.rs"]
mod preparation;

pub use application::WorthQueryWorkflowStageApplication;
pub use computation::{
    WorthQueryWorkflowStageComputationFailure, WorthQueryWorkflowStageComputePayload,
    WorthQueryWorkflowStageComputed, WorthQueryWorkflowStageTask,
};
pub use preparation::{
    WorthQueryWorkflowPreparationPredecessor, WorthQueryWorkflowStageInputFacts,
    WorthQueryWorkflowStagePreparation,
};

#[derive(Debug)]
pub enum WorthQueryWorkflowValue {
    NotRequired,
    Bool(bool),
    I64(i64),
    U64(u64),
    Text(String),
    EntityIdentity(String),
    CurrentEntityIdentity(crate::memory_workspace::WorthQueryEntityIdentity),
    Projection(Box<crate::ordinary::read::WorthQueryReadCompletion>),
    InstalledArtifact(crate::domain_installation::WorthQueryMoveOnlyArtifactHandle),
    TransferredArtifact(crate::domain_installation::WorthQueryTransferredArtifactHandle),
}

impl WorthQueryWorkflowValue {
    pub(crate) fn satisfies(
        &self,
        contract: &worth_query_installation::facade::WorthQueryWorkflowValueContract,
    ) -> bool {
        use worth_query_installation::facade::WorthQueryWorkflowValueContract as Contract;
        if let Contract::InstalledArtifact(reference) = contract {
            return match self {
                Self::InstalledArtifact(handle) => handle.contract_matches(reference),
                Self::TransferredArtifact(handle) => handle.contract_matches(reference),
                _ => false,
            };
        }
        matches!(
            (self, contract),
            (Self::NotRequired, Contract::NotRequired)
                | (Self::Bool(_), Contract::Bool)
                | (Self::I64(_), Contract::I64)
                | (Self::U64(_), Contract::U64)
                | (Self::Text(_), Contract::Text)
                | (Self::EntityIdentity(_), Contract::EntityIdentity)
                | (Self::CurrentEntityIdentity(_), Contract::EntityIdentity)
                | (Self::Projection(_), Contract::Projection)
        )
    }

    pub fn installed_artifact(
        handle: crate::domain_installation::WorthQueryMoveOnlyArtifactHandle,
    ) -> Self {
        Self::InstalledArtifact(handle)
    }

    pub fn into_transferred_artifact(
        self,
    ) -> Result<
        crate::domain_installation::WorthQueryTransferredArtifactHandle,
        WorthQueryWorkflowValue,
    > {
        match self {
            Self::TransferredArtifact(handle) => Ok(handle),
            value => Err(value),
        }
    }

    pub(crate) fn into_move_only_artifact(
        self,
    ) -> Result<crate::domain_installation::WorthQueryMoveOnlyArtifactHandle, WorthQueryWorkflowValue>
    {
        match self {
            Self::InstalledArtifact(handle) => Ok(handle),
            value => Err(value),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowStageWarning {
    Advisory(String),
    Partial(String),
}

pub struct WorthQueryWorkflowStageMaterial {
    output: WorthQueryWorkflowValue,
    warnings: Vec<WorthQueryWorkflowStageWarning>,
    result_state: Option<crate::domain_installation::WorthQueryOperationResultState>,
    primary_graph_reads: Vec<WorthQueryWorkflowPrimaryReadEvidence>,
    effects: Vec<WorthQueryWorkflowEffectEvidence>,
    executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    lineage: Vec<crate::identity_evolution::InstalledIdentityEvolutionOutcome>,
    domain_evidence: Option<super::WorthQueryDomainEvidenceMaterial>,
}

pub(crate) struct WorthQueryWorkflowStageMaterialParts {
    pub(crate) output: WorthQueryWorkflowValue,
    pub(crate) warnings: Vec<WorthQueryWorkflowStageWarning>,
    pub(crate) result_state: Option<crate::domain_installation::WorthQueryOperationResultState>,
    pub(crate) primary_graph_reads: Vec<WorthQueryWorkflowPrimaryReadEvidence>,
    pub(crate) effects: Vec<WorthQueryWorkflowEffectEvidence>,
    pub(crate) executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    pub(crate) lineage: Vec<crate::identity_evolution::InstalledIdentityEvolutionOutcome>,
    pub(crate) domain_evidence: Option<super::WorthQueryDomainEvidenceMaterial>,
}

impl WorthQueryWorkflowStageMaterial {
    pub fn new(output: WorthQueryWorkflowValue) -> Self {
        Self {
            output,
            warnings: Vec::new(),
            result_state: None,
            primary_graph_reads: Vec::new(),
            effects: Vec::new(),
            executed_effects: Vec::new(),
            lineage: Vec::new(),
            domain_evidence: None,
        }
    }

    pub fn with_primary_graph_read(
        mut self,
        role: impl Into<String>,
        completion: &crate::ordinary::read::WorthQueryReadCompletion,
    ) -> Self {
        self.primary_graph_reads
            .push(WorthQueryWorkflowPrimaryReadEvidence::from_completion(
                role, completion,
            ));
        self
    }

    pub fn projection(
        role: impl Into<String>,
        completion: crate::ordinary::read::WorthQueryReadCompletion,
    ) -> Self {
        let evidence = WorthQueryWorkflowPrimaryReadEvidence::from_completion(role, &completion);
        Self {
            output: WorthQueryWorkflowValue::Projection(Box::new(completion)),
            warnings: Vec::new(),
            result_state: None,
            primary_graph_reads: vec![evidence],
            effects: Vec::new(),
            executed_effects: Vec::new(),
            lineage: Vec::new(),
            domain_evidence: None,
        }
    }

    pub fn with_warning(mut self, warning: WorthQueryWorkflowStageWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_result_state(
        mut self,
        result_state: crate::domain_installation::WorthQueryOperationResultState,
    ) -> Self {
        self.result_state = Some(result_state);
        self
    }

    pub fn with_lineage_outcomes(
        mut self,
        lineage: Vec<crate::identity_evolution::InstalledIdentityEvolutionOutcome>,
    ) -> Self {
        self.lineage = lineage;
        self
    }

    pub fn with_domain_evidence(
        mut self,
        evidence: super::WorthQueryDomainEvidenceMaterial,
    ) -> Self {
        self.domain_evidence = Some(evidence);
        self
    }

    pub(crate) fn into_parts(self) -> WorthQueryWorkflowStageMaterialParts {
        WorthQueryWorkflowStageMaterialParts {
            output: self.output,
            warnings: self.warnings,
            result_state: self.result_state,
            primary_graph_reads: self.primary_graph_reads,
            effects: self.effects,
            executed_effects: self.executed_effects,
            lineage: self.lineage,
            domain_evidence: self.domain_evidence,
        }
    }

    pub(crate) fn retain_query_executed_effects(
        &mut self,
        effects: Vec<WorthQueryWorkflowEffectEvidence>,
    ) {
        self.effects = effects.clone();
        self.executed_effects = effects;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowStageExecutorFailure {
    class: worth_query_installation::facade::WorthQueryOperationFailureClass,
    detail: String,
    executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
}

impl WorthQueryWorkflowStageExecutorFailure {
    pub fn new(
        class: worth_query_installation::facade::WorthQueryOperationFailureClass,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            class,
            detail: detail.into(),
            executed_effects: Vec::new(),
        }
    }
    pub fn class(&self) -> &worth_query_installation::facade::WorthQueryOperationFailureClass {
        &self.class
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub(crate) fn with_executed_effects(
        mut self,
        executed_effects: Vec<WorthQueryWorkflowEffectEvidence>,
    ) -> Self {
        self.executed_effects = executed_effects;
        self
    }

    pub(crate) fn executed_effects(&self) -> &[WorthQueryWorkflowEffectEvidence] {
        &self.executed_effects
    }
}

/// One staged contract: inert preparation/computation, then canonical owner application.
/// Workspace-bound executors implement only `apply`; both inert phases default.
/// Prepare and compute must be pure functions of their given values: no clock,
/// static or global state, environment, interior-mutable state shared with the
/// owner or another member, or effects. Their types remove the executor, stage
/// execution context, and workspace; the meter exposes checkpoints only. They
/// cannot remove ambient state. Violating this obligation
/// makes results depend on execution order and worker count.
/// Preparation stops at the first canonical preparation failure. Later members
/// are neither prepared nor computed; earlier members compute exactly once.
/// Application runs the canonical prefix through the least failure across all
/// phases, preserving a failing application's partial owner effects.
/// The map charges the canonical compute prefix through its least failure.
/// A later owner-side apply failure does not refund computation already settled.
/// Under sufficient memory, worker width changes neither results nor charged
/// work when interruption is absent or already present at dispatch. Equality
/// excludes cancellation arriving during compute, deadlines arriving during
/// compute, and marginal memory admission. During interruption, application is
/// a canonical prefix with serial-equivalent receipts and effects; the boundary
/// is at or before the interrupting member, the cause stays cancellation or
/// deadline, and the map charges its canonical prefix through that boundary.
/// Real computation must checkpoint its meter; identity compute charges nothing.
/// Compute must not start nested work at any posture, including bounded serial:
/// the meter offers no descendant door. A nested stop is a contract failure.
///
/// The combined execution method no longer exists:
/// ```
/// use worth_query::facade::domain::*;
/// fn old<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, input: WorthQueryWorkflowValue,
///     context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
///     application: WorthQueryWorkflowStageApplication<'_, '_, '_>,
/// ) { let _ = executor.apply(application); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn old<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, input: WorthQueryWorkflowValue,
///     context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
///     application: WorthQueryWorkflowStageApplication<'_, '_, '_>,
/// ) { let _ = executor.execute_stage(input, context, workspace); }
/// ```
/// Preparation cannot receive execution authority:
/// ```
/// use worth_query::facade::domain::*;
/// fn leak<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     view: WorthQueryWorkflowStagePreparation<'_>,
/// ) { let _ = E::prepare(view); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn leak<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, context: &WorthQueryWorkflowStageExecutionContext<'_>,
///     view: WorthQueryWorkflowStagePreparation<'_>,
/// ) { let _ = E::prepare(context); }
/// ```
/// The preparation function cannot receive an executor instance (including its
/// interior state); registration captures the associated function directly.
/// ```
/// use worth_query::facade::domain::*;
/// fn prepare_from_instance<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, view: WorthQueryWorkflowStagePreparation<'_>,
/// ) { let _ = E::prepare(view); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn prepare_from_instance<E: WorthQueryDomainWorkflowStageExecutor<(), (), ()>>(
///     executor: &E, view: WorthQueryWorkflowStagePreparation<'_>,
/// ) { let _ = executor.prepare(view); }
/// ```
pub trait WorthQueryDomainWorkflowStageExecutor<D, O, F>: Send + Sync + 'static {
    const LOWERING_FAMILY: &'static str;
    const DETERMINISTIC: bool;
    const IDEMPOTENT_STAGE_RETRY: bool = false;
    const EXECUTION_COST: crate::domain_installation::WorthQueryOperationCostClass;
    const RESULT_WIDTH_COST: crate::domain_installation::WorthQueryOperationCostClass;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = None;

    fn installed_read_declaration(
        &self,
    ) -> Option<&crate::ordinary::read::WorthQueryReadDeclaration> {
        None
    }

    fn execution_resource_support(
        &self,
    ) -> crate::domain_installation::WorthQueryExecutionResourceSupport;

    fn prepare(
        view: WorthQueryWorkflowStagePreparation<'_>,
    ) -> Result<WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputationFailure> {
        Ok(view.task(WorthQueryWorkflowStageComputePayload::Empty))
    }

    fn compute(
        task: WorthQueryWorkflowStageTask,
        _meter: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> WorthQueryWorkflowStageComputed {
        task.pass_through()
    }

    fn apply(
        &self,
        application: WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<WorthQueryWorkflowStageMaterial, WorthQueryWorkflowStageExecutorFailure>;
}

/// Domain-owned semantic comparison for an executor registered on the
/// certification replay lane. Keeping this separate from stage execution makes
/// comparator presence an installation-time fact rather than a post-effect
/// discovery.
pub trait WorthQueryDomainReplaySemanticComparator<D, O, F>: Send + Sync + 'static {
    fn compare_replay_semantics(
        &self,
        original: &crate::domain_installation::WorthQueryWorkflowTraceSemantics,
        replay: &crate::domain_installation::WorthQueryWorkflowTraceSemantics,
        noise: crate::domain_installation::WorthQueryReplayNoiseContract,
    ) -> crate::domain_installation::WorthQueryReplayComparison;
}
