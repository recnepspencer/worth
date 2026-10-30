//! One actual producer field comparison, completed before Ready and retained through retries.
pub(super) mod installation;
mod native;
use super::output_equivalence_contract::InstalledProducerOutputEquivalence;
use crate::domain_computation::primary_graph::{
    application_output_demand::PreparedReadyBacking,
    conditional_operation::WorthQueryConditionalTruthBasis,
    output_reuse::{retained_fact_denial, RetainedFactStop},
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphLayout,
};
#[cfg(test)]
pub(super) use native::dependency_matches_output;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::{
    branch::RelationalBranchBasisDescriptor,
    identity::{EntityId, KindId},
    runtime::RelationalFieldRevision,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeConditionalExecutionRequest,
    BridgeExactOutputComparison, BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerEvaluationIneligibility {
    OutputRoleUnavailable,
    BeforeFieldUnavailable,
    ConflictingBeforeField,
    NativeAfterUnavailable,
    ConflictingNativeAfter,
    NativeAfterAbsent,
    ReadinessPublicationMismatch,
    ReadinessFieldMismatch,
}
pub(in crate::domain_computation::primary_graph) struct ProducerOutputEvaluation {
    field: Arc<InstalledProducerOutputEquivalence>,
    role: Arc<str>,
    pub(in crate::domain_computation::primary_graph) state: ProducerOutputEvaluationState,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(in crate::domain_computation::primary_graph) enum ProducerOutputEvaluationState {
    Ineligible(WorthQueryProducerEvaluationIneligibility),
    CaptureUnavailable(
        crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure,
    ),
    Compared(ComparedProducerOutput),
}
pub(in crate::domain_computation::primary_graph) struct ComparedProducerOutput {
    scope: Arc<InstalledProducerOutputEquivalence>,
    pub(in crate::domain_computation::primary_graph) role: usize,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) revision: RelationalFieldRevision,
    pub(in crate::domain_computation::primary_graph) performed_basis:
        RelationalBranchBasisDescriptor,
    pub(in crate::domain_computation::primary_graph) comparison: BridgeExactOutputComparison,
}
impl ComparedProducerOutput {
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus {
        &self.scope.field.binding.locus
    }
    pub(in crate::domain_computation::primary_graph) fn contract_revision(
        &self,
    ) -> worth_foundational::facade::AspectContractRevision {
        self.scope.field.aspect.revision()
    }
    pub(in crate::domain_computation::primary_graph) fn field_meaning(&self) -> [u8; 32] {
        self.scope.meaning
    }
}

impl ProducerOutputEvaluation {
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus {
        &self.field.field.binding.locus
    }
    pub(in crate::domain_computation::primary_graph) fn role(&self) -> &str {
        &self.role
    }
}

/// One installed comparison owns its exact field/role meaning and lowering.
pub(in crate::domain_computation::primary_graph) struct InstalledOutputComparison {
    lowering: Arc<BridgeInstalledConditionalLowering>,
    field: Arc<InstalledProducerOutputEquivalence>,
    role: Arc<str>,
    producer: String,
}
/// The value and dependency metadata come from this one actual readiness execution.
pub(super) struct NativeOutputBasis<'a> {
    pub(super) receipt: &'a WorthQueryApplicationCommitReceipt,
    pub(super) readiness: &'a BridgeConditionalDecisionEvidence,
    pub(super) lowering: &'a BridgeInstalledConditionalLowering,
    pub(super) ordinal: usize,
    pub(super) layout: &'a WorthQueryPrimaryGraphLayout,
}
/// Visit metadata is used only to target the certification interruption seam.
/// The original comparison remains the sole owner of evaluation evidence.
pub(super) struct OutputEvaluationVisit {
    pub(super) evaluation: Arc<ProducerOutputEvaluation>,
    pub(super) compared_now: bool,
}
impl InstalledOutputComparison {
    pub(super) fn evaluate(
        &self,
        execution: ExecutionRequest<'_, '_>,
        bridge: &BridgeSealedRuntimeAssembly,
        truth: &WorthQueryConditionalTruthBasis,
        basis: NativeOutputBasis<'_>,
        ready: &PreparedReadyBacking,
        attempts: &std::sync::atomic::AtomicU64,
    ) -> Result<OutputEvaluationVisit, WorthQueryOutputDemandDenial> {
        consult(&execution)?;
        if let Some(existing) = ready.output_evaluation() {
            return Ok(OutputEvaluationVisit {
                evaluation: Arc::clone(existing),
                compared_now: false,
            });
        }
        let header = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<ProducerOutputEvaluation>())
            .expect("fixed evaluation header")
            .0
            .pad_to_align();
        let memory = execution
            .reserve_retained_memory(header.size() as u64)
            .map_err(|cause| {
                retained_fact_denial(
                    RetainedFactStop::Execution(cause.into()),
                    "producer output evaluation",
                )
            })?;
        let state = if let Some(Err(cause)) = basis.receipt.performed_output_fields() {
            ProducerOutputEvaluationState::CaptureUnavailable(cause.clone())
        } else {
            match native::prepare(
                &execution,
                &self.field,
                &self.role,
                &basis,
                truth.is_publication(basis.receipt),
            )? {
                Err(cause) => ProducerOutputEvaluationState::Ineligible(cause),
                Ok(observed) => {
                    let attempt = attempts
                        .fetch_update(
                            std::sync::atomic::Ordering::Relaxed,
                            std::sync::atomic::Ordering::Relaxed,
                            |current| current.checked_add(2),
                        )
                        .ok()
                        .and_then(|baseline| baseline.checked_add(1))
                        .ok_or_else(|| {
                            WorthQueryOutputDemandDenial::new(
                                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                                "output comparison attempt exhausted",
                            )
                        })?;
                    let signal = bridge
                        .admit_exact_conditional_signal_basis(&self.lowering, truth.signal_basis())
                        .map_err(|cause| {
                            super::demand::bridge_denial::bridge_denial(&self.producer, cause)
                        })?;
                    let comparison = bridge
                        .compare_exact_output(
                            execution,
                            &signal,
                            BridgeConditionalExecutionRequest {
                                lowering: &self.lowering,
                                query_binding_identity: &self.producer,
                                query_capability_identity: attempt,
                                snapshot_identity: truth.snapshot_projection(),
                                truth_branch_identity: Some(truth.branch_projection()),
                                bridge_snapshot_identity: None,
                                execution_identity: &self.producer,
                                attempt,
                            },
                            observed.before,
                            observed.after,
                        )
                        .map_err(WorthQueryOutputDemandDenial::output_comparison)?;
                    ProducerOutputEvaluationState::Compared(ComparedProducerOutput {
                        scope: Arc::clone(&self.field),
                        role: observed.role,
                        entity: observed.entity,
                        kind: observed.kind,
                        revision: observed.revision,
                        performed_basis: basis.receipt.basis_descriptor().clone(),
                        comparison,
                    })
                }
            }
        };
        // No fallible stage remains between a completed comparison and its memo.
        let completed = Arc::new(ProducerOutputEvaluation {
            field: Arc::clone(&self.field),
            role: Arc::clone(&self.role),
            state,
            _memory: memory,
        });
        ready.retain_output_evaluation(Arc::clone(&completed));
        let compared_now = matches!(completed.state, ProducerOutputEvaluationState::Compared(_));
        Ok(OutputEvaluationVisit {
            evaluation: completed,
            compared_now,
        })
    }
}
pub(super) fn consult(
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    execution.consult().map_err(|cause| {
        retained_fact_denial(
            RetainedFactStop::Execution(cause.into()),
            "producer output evaluation",
        )
    })
}
