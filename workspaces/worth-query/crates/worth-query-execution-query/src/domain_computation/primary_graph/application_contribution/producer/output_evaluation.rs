//! One actual producer field comparison, completed before Ready and retained through retries.
use crate::domain_computation::primary_graph::{
    conditional_operation::installation::route_binding::InstalledConditionalRoute,
    managed_bridge::conditional_execution::ConditionalExecutionOwner,
};
pub mod installation;
pub mod native;
use super::output_equivalence_contract::InstalledProducerOutputEquivalence;
use crate::domain_computation::primary_graph::{
    application_output_demand::PreparedReadyBacking,
    conditional_operation::WorthQueryConditionalTruthBasis,
    output_reuse::{retained_fact_denial, RetainedFactStop},
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphLayout,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeConditionalExecutionRequest,
    BridgeInstalledConditionalLowering,
};

pub use worth_query_execution_lineage::facade::primary_graph::application_contribution::producer::output_evaluation::{
    ComparedProducerOutput, ProducerOutputEvaluation, ProducerOutputEvaluationState,
    WorthQueryProducerEvaluationIneligibility,
};

/// One installed comparison owns its exact field/role meaning and lowering.
pub struct InstalledOutputComparison {
    lowering: InstalledConditionalRoute,
    field: Arc<InstalledProducerOutputEquivalence>,
    role: Arc<str>,
    producer: String,
}
impl InstalledOutputComparison {
    pub fn admit_authored_declaration(
        &mut self,
        fresh: &worth_runtime_bridge::facade::BridgeFreshAuthoredBindings,
    ) -> Result<(), worth_runtime_bridge::facade::BridgeAuthoredReadmissionDenial> {
        self.lowering.admit_authored_declaration(fresh)
    }
}

/// The value and dependency metadata come from this one actual readiness execution.
pub struct NativeOutputBasis<'a> {
    pub receipt:
        crate::domain_computation::primary_graph::application_output_demand::OutputPublicationView<
            'a,
        >,
    pub readiness: &'a BridgeConditionalDecisionEvidence,
    pub lowering: &'a BridgeInstalledConditionalLowering,
    pub ordinal: usize,
    pub layout: &'a WorthQueryPrimaryGraphLayout,
}
/// Visit metadata is used only to target the certification interruption seam.
/// The original comparison remains the sole owner of evaluation evidence.
pub struct OutputEvaluationVisit {
    pub evaluation: Arc<ProducerOutputEvaluation>,
    pub compared_now: bool,
}
impl InstalledOutputComparison {
    pub fn evaluate(
        &self,
        execution: ExecutionRequest<'_, '_>,
        bridge: &ConditionalExecutionOwner<'_>,
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
                truth.is_product_publication(basis.receipt.committed_product_publication()),
            )? {
                Err(cause) => ProducerOutputEvaluationState::Ineligible(cause),
                Ok(observed) => {
                    let attempt = next_comparison_attempt(attempts).ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::SchedulingRejected,
                            "output comparison attempt exhausted",
                        )
                    })?;
                    let signal = self
                        .lowering
                        .admit_signal_basis(bridge, truth.signal_basis())
                        .map_err(|cause| {
                            super::demand::bridge_denial::bridge_denial(&self.producer, cause)
                        })?;
                    let comparison = bridge
                        .compare_exact_output(
                            execution,
                            &signal,
                            BridgeConditionalExecutionRequest {
                                lowering: signal.installed_lowering_ref(),
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
                    ProducerOutputEvaluationState::Compared(ComparedProducerOutput::new(
                        Arc::clone(&self.field),
                        observed.role,
                        observed.entity,
                        observed.kind,
                        observed.revision,
                        basis.receipt.basis_descriptor().clone(),
                        comparison,
                    ))
                }
            }
        };
        // No fallible stage remains between a completed comparison and its memo.
        let completed = Arc::new(ProducerOutputEvaluation::new(
            Arc::clone(&self.field),
            Arc::clone(&self.role),
            state,
            memory,
        ));
        ready.retain_output_evaluation(Arc::clone(&completed));
        let compared_now = matches!(completed.state, ProducerOutputEvaluationState::Compared(_));
        Ok(OutputEvaluationVisit {
            evaluation: completed,
            compared_now,
        })
    }
}
fn next_comparison_attempt(attempts: &std::sync::atomic::AtomicU64) -> Option<u64> {
    use std::sync::atomic::Ordering::Relaxed;
    let mut current = attempts.load(Relaxed);
    loop {
        let next = current.checked_add(2)?;
        match attempts.compare_exchange_weak(current, next, Relaxed, Relaxed) {
            Ok(baseline) => return baseline.checked_add(1),
            Err(observed) => current = observed,
        }
    }
}

pub fn consult(
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    execution.consult().map_err(|cause| {
        retained_fact_denial(
            RetainedFactStop::Execution(cause.into()),
            "producer output evaluation",
        )
    })
}

#[cfg(test)]
mod attempt_tests {
    use super::next_comparison_attempt;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    #[test]
    fn comparison_attempts_refuse_exhaustion_without_reusing_or_wrapping() {
        let attempts = AtomicU64::new(0);
        assert_eq!(next_comparison_attempt(&attempts), Some(1));
        assert_eq!(next_comparison_attempt(&attempts), Some(3));
        for exhausted in [u64::MAX - 1, u64::MAX] {
            attempts.store(exhausted, Relaxed);
            assert_eq!(next_comparison_attempt(&attempts), None);
            assert_eq!(attempts.load(Relaxed), exhausted);
        }
        attempts.store(u64::MAX - 2, Relaxed);
        assert_eq!(next_comparison_attempt(&attempts), Some(u64::MAX - 1));
        assert_eq!(attempts.load(Relaxed), u64::MAX);
        assert_eq!(next_comparison_attempt(&attempts), None);
    }
}
