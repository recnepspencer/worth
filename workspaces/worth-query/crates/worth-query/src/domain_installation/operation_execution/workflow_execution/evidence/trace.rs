use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryCompletedWorkflowPhase,
    WorthQueryOperationPhaseProof,
};
use crate::domain_installation::operation_identity_basis::canonical_indexed_operation_material;
use crate::identity::hash_parts;

use super::{WorthQueryWorkflowRun, WorthQueryWorkflowRunCounters, WorthQueryWorkflowStageReceipt};
use worth_proof::TransitionOutcome;

#[path = "trace/semantic_identity.rs"]
mod semantic_identity;

use semantic_identity::semantic_trace_identity;

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub fn complete(
        mut self,
    ) -> TransitionOutcome<
        WorthQueryCompletedWorkflowTrace<D, O, F, L>,
        WorthQueryWorkflowCompletionDenial,
        std::convert::Infallible,
        WorthQueryWorkflowCompletionDenial,
        WorthQueryWorkflowCompletionDenial,
        WorthQueryWorkflowCompletionDenial,
    > {
        if !self.bound.installation_is_current() {
            let denial = WorthQueryWorkflowCompletionDenial::from_run(
                WorthQueryWorkflowCompletionDenialKind::StaleInstallationGeneration,
                &self,
            );
            return TransitionOutcome::Stale(
                denial.with_managed_cleanup(cleanup_abandoned_workflow(&mut self)),
            );
        }
        if self.completed.len() != self.graph.stages().len() {
            let denial = WorthQueryWorkflowCompletionDenial::from_run(
                WorthQueryWorkflowCompletionDenialKind::IncompleteStages,
                &self,
            );
            return TransitionOutcome::Denied(
                denial.with_managed_cleanup(cleanup_abandoned_workflow(&mut self)),
            );
        }
        let mut run = self;
        for receipt in run.receipts.iter_mut().rev() {
            receipt.retire_artifact_output();
        }
        run.artifact_registry.close_released();
        let running = run
            .managed
            .take()
            .expect("completed workflow owns its managed run");
        let terminal = match running.completed() {
            Ok(terminal) => terminal,
            Err(rejection) => {
                let detail = rejection.denial().detail().to_owned();
                let cleanup = rejection.into_running().abandon().cleanup();
                return TransitionOutcome::Denied(
                    WorthQueryWorkflowCompletionDenial::from_run(
                        WorthQueryWorkflowCompletionDenialKind::ManagedRun,
                        &run,
                    )
                    .with_managed_detail(detail)
                    .with_managed_cleanup(cleanup),
                );
            }
        };
        match terminal.cleanup() {
            worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => {
                run.managed_cleanup = Some(receipt);
            }
            cleanup => {
                return TransitionOutcome::Denied(
                    WorthQueryWorkflowCompletionDenial::from_run(
                        WorthQueryWorkflowCompletionDenialKind::ManagedRun,
                        &run,
                    )
                    .with_managed_detail("managed workflow cleanup requires owner resolution")
                    .with_managed_cleanup(cleanup),
                );
            }
        }
        let mut trace = mint_completed_trace(run);
        match crate::domain_installation::dependency_impact::compile_workflow_semantic_aspect_dependencies(&trace) {
            Ok(dependency_closure) => trace.dependency_closure = Some(dependency_closure),
            Err(denial) => {
                return TransitionOutcome::Denied(WorthQueryWorkflowCompletionDenial::from_trace(
                    WorthQueryWorkflowCompletionDenialKind::DependencyCompilation(denial),
                    &trace,
                ));
            }
        }
        match crate::domain_installation::operation_lineage::bind_execution_lineage(trace) {
            Ok(trace) => TransitionOutcome::Success(trace),
            Err((trace, _)) => {
                TransitionOutcome::Denied(WorthQueryWorkflowCompletionDenial::from_trace(
                    WorthQueryWorkflowCompletionDenialKind::LineageEvidence,
                    &trace,
                ))
            }
        }
    }
}

fn cleanup_abandoned_workflow<D, O, F, L: BasisOperationLane>(
    run: &mut WorthQueryWorkflowRun<D, O, F, L>,
) -> worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome {
    for receipt in run.receipts.iter_mut().rev() {
        receipt.cancel_artifact_output();
    }
    run.artifact_registry.close_cancelled();
    run.managed
        .take()
        .expect("live workflow owns its managed run")
        .abandon()
        .cleanup()
}

fn mint_completed_trace<D, O, F, L: BasisOperationLane>(
    run: WorthQueryWorkflowRun<D, O, F, L>,
) -> WorthQueryCompletedWorkflowTrace<D, O, F, L> {
    let mut receipt_identities = run
        .receipts
        .iter()
        .map(|receipt| (receipt.stage_identity(), receipt.identity()))
        .collect::<Vec<_>>();
    receipt_identities.sort();
    let operation_conditional = canonical_indexed_operation_material(
        "workflow.operation.conditional",
        run.operation_conditional_provenance()
            .iter()
            .map(super::workflow_conditional_trace::conditional_trace_operational_material),
    );
    let identity = hash_parts(&[
        "worth_query_completed_workflow_trace_v1".into(),
        format!("run:{}", run.identity),
        format!("operation_conditional:{operation_conditional}"),
        format!(
            "receipts:{}",
            receipt_identities
                .iter()
                .map(|(_, identity)| *identity)
                .collect::<Vec<_>>()
                .join(",")
        ),
    ]);
    let semantic_identity = semantic_trace_identity(&run);
    let phase_proof = mint_operation_phase_proof(
        identity.clone(),
        Some(run.authority_proof.proof.payload().identity()),
        operation_phase_basis(&run.authority_proof.proof).clone(),
    );
    WorthQueryCompletedWorkflowTrace {
        run,
        identity,
        semantic_identity,
        phase_proof,
        lineage: None,
        dependency_closure: None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowCompletionDenialKind {
    StaleInstallationGeneration,
    IncompleteStages,
    LineageEvidence,
    DependencyCompilation(
        crate::domain_installation::WorthQuerySemanticAspectDependencyCompilationDenial,
    ),
    ManagedRun,
}

#[derive(Debug)]
pub struct WorthQueryWorkflowCompletionDenial {
    kind: WorthQueryWorkflowCompletionDenialKind,
    executed_effects: Vec<super::WorthQueryWorkflowEffectEvidence>,
    counters: WorthQueryWorkflowRunCounters,
    managed_detail: Option<String>,
    managed_cleanup:
        Option<worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome>,
}

impl WorthQueryWorkflowCompletionDenial {
    fn from_run<D, O, F, L: BasisOperationLane>(
        kind: WorthQueryWorkflowCompletionDenialKind,
        run: &WorthQueryWorkflowRun<D, O, F, L>,
    ) -> Self {
        Self {
            kind,
            executed_effects: run
                .receipts
                .iter()
                .flat_map(|receipt| receipt.effect_evidence().iter().cloned())
                .collect(),
            counters: run.counters,
            managed_detail: None,
            managed_cleanup: None,
        }
    }

    fn from_trace<D, O, F, L: BasisOperationLane>(
        kind: WorthQueryWorkflowCompletionDenialKind,
        trace: &WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    ) -> Self {
        Self::from_run(kind, &trace.run)
    }

    pub const fn kind(&self) -> WorthQueryWorkflowCompletionDenialKind {
        self.kind
    }

    pub fn executed_effects(&self) -> &[super::WorthQueryWorkflowEffectEvidence] {
        &self.executed_effects
    }

    pub const fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.counters
    }

    pub fn managed_detail(&self) -> Option<&str> {
        self.managed_detail.as_deref()
    }

    pub fn take_managed_cleanup(
        &mut self,
    ) -> Option<worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome> {
        self.managed_cleanup.take()
    }

    fn with_managed_detail(mut self, detail: impl Into<String>) -> Self {
        self.managed_detail = Some(detail.into());
        self
    }

    fn with_managed_cleanup(
        mut self,
        cleanup: worth_query_execution::facade::runtime::WorthQueryWorkflowRunCleanupOutcome,
    ) -> Self {
        self.managed_cleanup = Some(cleanup);
        self
    }
}

pub struct WorthQueryCompletedWorkflowTrace<D, O, F, L: BasisOperationLane> {
    pub(super) run: WorthQueryWorkflowRun<D, O, F, L>,
    pub(super) identity: String,
    semantic_identity: String,
    pub(super) phase_proof: WorthQueryOperationPhaseProof<WorthQueryCompletedWorkflowPhase>,
    pub(crate) lineage: Option<crate::domain_installation::WorthQueryTraceLineageReport>,
    dependency_closure:
        Option<crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryCompletedWorkflowTrace<D, O, F, L> {
    pub(crate) fn bound(
        &self,
    ) -> &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L> {
        &self.run.bound
    }
    pub(crate) fn phase_proof(
        &self,
    ) -> &WorthQueryOperationPhaseProof<WorthQueryCompletedWorkflowPhase> {
        &self.phase_proof
    }
    pub(crate) fn workflow_run_identity(&self) -> &str {
        self.run.identity()
    }
    pub(crate) fn installed_workflow_read(
        &self,
    ) -> Option<&crate::ordinary::read::WorthQueryReadDeclaration> {
        self.run.executor.installed_read.as_ref()
    }
    pub fn identity(&self) -> &str {
        debug_assert_eq!(self.phase_proof.payload().identity(), self.identity);
        &self.identity
    }
    pub fn semantic_identity(&self) -> &str {
        &self.semantic_identity
    }
    pub fn stage_receipts(&self) -> &[WorthQueryWorkflowStageReceipt] {
        &self.run.receipts
    }
    pub fn operation_conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        self.run.operation_conditional_provenance()
    }
    pub fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.run.counters
    }
    pub fn resources(&self) -> &crate::domain_installation::WorthQueryAdmittedWorkflowResourcePlan {
        self.run.resources()
    }
    pub fn operation_resource_evidence(
        &self,
    ) -> &crate::domain_installation::WorthQueryExecutionResourceAttemptEvidence {
        self.run.operation_resource_evidence()
    }
    pub fn lineage_report(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryTraceLineageReport> {
        self.lineage.as_ref()
    }
    pub fn semantic_aspect_dependency_closure(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>
    {
        self.dependency_closure.as_ref()
    }
    pub fn classify_authoritative_impact(
        &self,
        delivery: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
        conditional: &crate::domain_installation::WorthQueryConditionalProvenance,
    ) -> Result<
        crate::domain_installation::WorthQueryImpactDecision,
        crate::domain_installation::WorthQueryImpactAdmissionDenial,
    > {
        let closure = self.semantic_aspect_dependency_closure().ok_or_else(|| {
            crate::domain_installation::WorthQueryImpactAdmissionDenial::new(
                crate::domain_installation::WorthQueryImpactAdmissionDenialKind::DependencyClosureUnavailable,
                crate::domain_installation::WorthQueryImpactCounters::default(),
            )
        })?;
        crate::domain_installation::classify_owner_delivered_impact(closure, delivery, conditional)
    }
    pub(crate) fn refresh_semantic_identity_for_lineage(&mut self) {
        let Some(lineage) = &self.lineage else {
            return;
        };
        self.semantic_identity = hash_parts(&[
            "worth_query_workflow_semantic_trace_with_lineage_v1".into(),
            format!("workflow:{}", self.semantic_identity),
            format!("lineage:{}", lineage.semantic_part()),
        ]);
    }
}
