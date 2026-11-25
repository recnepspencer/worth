use worth_runtime_bridge::facade::BridgeConditionalExecutionView;
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeManagedConditionalExecutionRequest,
    BridgeManagedDueWake,
};

use super::predicate_admission::QueryConditionalComputeContext;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis;

mod retained_decision;
pub(super) use retained_decision::{
    WorthQueryOperationBackpressureCause, WorthQueryRetainedConditionalDecision,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryConditionalContinuationIdentity {
    pub(super) key: [u8; 32],
    pub(super) intent: [u8; 32],
    pub(super) product: Option<worth_foundational::facade::DurableProductBranchIncarnation>,
}

pub(super) struct WorthQueryRetainedConditionalWake {
    pub(super) lifecycle_token: std::sync::Arc<()>,
    pub(super) due: BridgeManagedDueWake,
    pub(super) decision: WorthQueryRetainedConditionalDecision,
    pub(super) attempt: u64,
    pub(super) last_signal_decision: Option<super::WorthQueryConditionalSignalDecision>,
    pub(super) application_attempted: bool,
    pub(super) continuation_identity: Option<WorthQueryConditionalContinuationIdentity>,
    /// Explicit settlement state recorded by `retry_original_key`:
    /// `None` = unresolved, `Some(false)` = resolved in World but durable
    /// `Finish` was refused and must retry, `Some(true)` = durable `Finish`
    /// confirmed.
    pub(super) original_retry_finish_confirmed: Option<bool>,
    pub(super) application_admission_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

#[derive(Clone, Copy, Default)]
pub(super) struct WorthQueryConditionalDecisionCounts {
    pub(super) eligible: usize,
    pub(super) suppressed: usize,
    pub(super) deferred: usize,
    pub(super) failed: usize,
}

pub(super) fn retained_decision_counts(
    wakes: &[WorthQueryRetainedConditionalWake],
) -> WorthQueryConditionalDecisionCounts {
    let mut counts = WorthQueryConditionalDecisionCounts::default();
    for wake in wakes {
        let _intent_revision = wake.due.revision();
        match &wake.decision {
            WorthQueryRetainedConditionalDecision::OperationCleanupRequired(evidence, cleanup) => {
                let _ = (evidence, cleanup);
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationPreparationPending(
                evidence,
                pending,
            ) => {
                let _ = (evidence, pending);
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationPreparationRefused(
                evidence,
                refused,
            ) => {
                let _ = (evidence, refused);
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationProductStale(evidence, stale) => {
                let _decision = evidence.signal().class();
                let _expected_product = stale.expected_product();
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationNoEffect(evidence, cause) => {
                let _decision = evidence.signal().class();
                let _typed_no_effect_cause = cause;
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::Eligible(evidence) => {
                let _decision = evidence.signal().class();
                counts.eligible += 1;
            }
            WorthQueryRetainedConditionalDecision::Suppressed(evidence) => {
                let _decision = evidence.signal().class();
                counts.suppressed += 1;
            }
            WorthQueryRetainedConditionalDecision::Deferred(evidence) => {
                let _decision = evidence.signal().class();
                counts.deferred += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationBackpressured(evidence, cause) => {
                let _decision = evidence.signal().class();
                let _typed_backpressure_cause = cause;
                counts.deferred += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationControlStopped(evidence, cause) => {
                let _decision = evidence.signal().class();
                let _typed_control_stop = cause;
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationTerminalFailure(evidence, cause) => {
                let _decision = evidence.signal().class();
                let _typed_terminal_failure = cause;
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationProductUnpublished(
                evidence,
                custody,
                _,
            ) => {
                let _decision = evidence.signal().class();
                let _custody = custody;
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationExecutionControlRetryable(
                evidence,
                kind,
            ) => {
                let _ = (evidence, kind);
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationCommitRetryable(evidence, kind) => {
                let _ = (evidence, kind);
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail)
            | WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, detail) => {
                let _decision = evidence.signal().class();
                let _failure_cause = detail;
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
            | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_, _)
            | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(_) => {}
            WorthQueryRetainedConditionalDecision::InterruptedDuringReentry => {
                counts.failed += 1;
            }
            WorthQueryRetainedConditionalDecision::Failed(cause) => {
                let _failure_kind = cause.kind();
                counts.failed += 1;
            }
        }
    }
    counts
}

pub(super) fn evaluate_due_wake(
    execution: worth_execution::ExecutionRequest<'_, '_>,

    bridge: &BridgeConditionalExecutionView<'_>,
    due: BridgeManagedDueWake,
    signal_basis: &worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding,
    query_binding_identity: &str,
    query_capability_identity: u64,
    truth: &WorthQueryConditionalTruthBasis,
    triggering_correspondence: Option<
        &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    >,
) -> WorthQueryRetainedConditionalWake {
    let attempt = due.signal_ready_ordinal();
    let mut compute = QueryConditionalComputeContext {
        output_version: attempt,
    };
    let lowering = signal_basis.installed_lowering_ref();
    let result = bridge.execute_managed_due_wake(
        execution,
        BridgeManagedConditionalExecutionRequest {
            due_wake: &due,
            lowering,
            signal_basis,
            query_binding_identity,
            query_capability_identity,
            snapshot_identity: truth.snapshot_projection(),
            truth_branch_identity: Some(truth.branch_projection()),
            bridge_snapshot_identity: Some(truth.snapshot()),
            triggering_correspondence,
            attempt,
        },
        &mut compute,
    );
    let last_signal_decision = result
        .as_ref()
        .ok()
        .map(|evidence| super::execution_provenance::signal_decision(evidence.signal().class()));
    let decision = match result {
        Ok(evidence) => classify(evidence),
        Err(denial) => WorthQueryRetainedConditionalDecision::Failed(denial),
    };
    WorthQueryRetainedConditionalWake {
        lifecycle_token: Default::default(),
        due,
        decision,
        attempt,
        last_signal_decision,
        application_attempted: false,
        continuation_identity: None,
        original_retry_finish_confirmed: None,
        application_admission_canonical_work:
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
    }
}

pub(super) fn reconsider_retained_wake(
    execution: worth_execution::ExecutionRequest<'_, '_>,

    bridge: &BridgeConditionalExecutionView<'_>,
    wake: &mut WorthQueryRetainedConditionalWake,
    signal_basis: &worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding,
    query_binding_identity: &str,
    query_capability_identity: u64,
    truth: &WorthQueryConditionalTruthBasis,
    triggering_correspondence: Option<
        &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    >,
) {
    if !matches!(
        wake.decision,
        WorthQueryRetainedConditionalDecision::Suppressed(_)
            | WorthQueryRetainedConditionalDecision::Deferred(_)
    ) {
        return;
    }
    wake.attempt = wake.attempt.saturating_add(1);
    let mut compute = QueryConditionalComputeContext {
        output_version: wake.attempt,
    };
    let lowering = signal_basis.installed_lowering_ref();
    let result = bridge.execute_managed_due_wake(
        execution,
        BridgeManagedConditionalExecutionRequest {
            due_wake: &wake.due,
            lowering,
            signal_basis,
            query_binding_identity,
            query_capability_identity,
            snapshot_identity: truth.snapshot_projection(),
            truth_branch_identity: Some(truth.branch_projection()),
            bridge_snapshot_identity: Some(truth.snapshot()),
            triggering_correspondence,
            attempt: wake.attempt,
        },
        &mut compute,
    );
    wake.last_signal_decision = result
        .as_ref()
        .ok()
        .map(|evidence| super::execution_provenance::signal_decision(evidence.signal().class()));
    wake.application_attempted = false;
    wake.application_admission_canonical_work =
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero();
    wake.decision = match result {
        Ok(evidence) => classify(evidence),
        Err(denial) => WorthQueryRetainedConditionalDecision::Failed(denial),
    };
}

pub(super) fn reconsider_execution_denied_wake(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    bridge: &BridgeConditionalExecutionView<'_>,
    wake: &mut WorthQueryRetainedConditionalWake,
    signal_basis: &worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding,
    query_binding_identity: &str,
    query_capability_identity: u64,
    truth: &WorthQueryConditionalTruthBasis,
    triggering_correspondence: Option<
        &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    >,
) {
    if !matches!(
        &wake.decision,
        WorthQueryRetainedConditionalDecision::Failed(denial)
            if matches!(
                denial.kind(),
                worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(_)
            )
    ) {
        return;
    }
    wake.attempt = wake.attempt.saturating_add(1);
    let mut compute = QueryConditionalComputeContext {
        output_version: wake.attempt,
    };
    let lowering = signal_basis.installed_lowering_ref();
    let result = bridge.execute_managed_due_wake(
        execution,
        BridgeManagedConditionalExecutionRequest {
            due_wake: &wake.due,
            lowering,
            signal_basis,
            query_binding_identity,
            query_capability_identity,
            snapshot_identity: truth.snapshot_projection(),
            truth_branch_identity: Some(truth.branch_projection()),
            bridge_snapshot_identity: Some(truth.snapshot()),
            triggering_correspondence,
            attempt: wake.attempt,
        },
        &mut compute,
    );
    wake.last_signal_decision = result
        .as_ref()
        .ok()
        .map(|evidence| super::execution_provenance::signal_decision(evidence.signal().class()));
    wake.application_attempted = false;
    wake.application_admission_canonical_work =
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero();
    wake.decision = match result {
        Ok(evidence) => classify(evidence),
        Err(denial) => WorthQueryRetainedConditionalDecision::Failed(denial),
    };
}

fn classify(evidence: BridgeConditionalDecisionEvidence) -> WorthQueryRetainedConditionalDecision {
    match classify_bridge_signal(&evidence) {
        super::WorthQueryConditionalSignalDecision::Eligible => {
            WorthQueryRetainedConditionalDecision::Eligible(evidence)
        }
        super::WorthQueryConditionalSignalDecision::DependencyUnchanged
        | super::WorthQueryConditionalSignalDecision::RevertedClean
        | super::WorthQueryConditionalSignalDecision::Suppressed => {
            WorthQueryRetainedConditionalDecision::Suppressed(evidence)
        }
        super::WorthQueryConditionalSignalDecision::Deferred => {
            WorthQueryRetainedConditionalDecision::Deferred(evidence)
        }
    }
}

pub(crate) fn classify_bridge_signal(
    evidence: &BridgeConditionalDecisionEvidence,
) -> super::WorthQueryConditionalSignalDecision {
    super::execution_provenance::signal_decision(evidence.signal().class())
}

pub(crate) fn classify_bridge_signal_observation(
    evidence: &worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
) -> super::WorthQueryConditionalSignalDecision {
    super::execution_provenance::signal_decision(evidence.signal_class())
}
