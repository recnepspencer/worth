use std::sync::Arc;

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;
use worth_query_admission::facade::resource_admission::WorthQueryFixedExecutionCapacity;
use worth_query_declaration::facade::domain_computation::WorthQueryCancellationSafePointFamily;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionMode;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceLimitRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryRetainedProgressPosture;
use worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryYieldedStatePosture;
use worth_query_installation::facade::WorthQueryExecutionAccessProductFamily;
use worth_query_installation::facade::WorthQueryExecutionAllocatorFamily;
use worth_query_installation::facade::WorthQueryExecutionProviderFamily;
use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;

pub fn execution_support(yieldable: bool) -> WorthQueryExecutionResourceSupport {
    execution_support_with_limit(8, yieldable)
}

pub fn execution_support_with_limit(
    limit: u64,
    yieldable: bool,
) -> WorthQueryExecutionResourceSupport {
    let envelope = execution_envelope(
        WorthQuerySemanticScaleRequest::bounded(limit),
        WorthQueryResourceLimitRequest::bounded(limit)
            .with(WorthQueryResourceDimension::RetainedBytes, 4_096),
        yieldable,
    );
    execution_support_for_envelope(envelope, format!("convergence-capacity-{limit}"), limit)
}

pub fn execution_support_with_broader_stage_queue_contract() -> WorthQueryExecutionResourceSupport {
    let envelope = execution_envelope(
        WorthQuerySemanticScaleRequest::bounded(4),
        WorthQueryResourceLimitRequest::bounded(4)
            .with(WorthQueryResourceDimension::QueueDepth, 8)
            .with(WorthQueryResourceDimension::RetainedBytes, 4_096),
        false,
    );
    execution_support_for_envelope(envelope, "convergence-stage-queue-capacity", 8)
}

pub fn execution_envelope(
    scale: WorthQuerySemanticScaleRequest,
    limits: WorthQueryResourceLimitRequest,
    yieldable: bool,
) -> WorthQueryExecutionResourceEnvelope {
    let envelope = WorthQueryExecutionResourceEnvelope::new(
        scale,
        limits,
        WorthQueryExecutionMode::Synchronous,
        None,
        WorthQueryCancellationSafePointFamily::new("convergence-step").unwrap(),
    );
    if !yieldable {
        return envelope;
    }
    envelope
        .with_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint)
        .with_retained_progress_posture(WorthQueryRetainedProgressPosture::RetainAttemptCapacity)
}

pub fn execution_support_for_envelope(
    envelope: WorthQueryExecutionResourceEnvelope,
    capacity_identity: impl Into<Arc<str>>,
    capacity: u64,
) -> WorthQueryExecutionResourceSupport {
    WorthQueryExecutionResourceSupport::new(
        WorthQueryExecutionProviderFamily::new("convergence-provider").unwrap(),
        WorthQueryExecutionAccessProductFamily::new("convergence-access").unwrap(),
        WorthQueryExecutionAllocatorFamily::new("convergence-arena").unwrap(),
        envelope,
        Arc::new(
            WorthQueryFixedExecutionCapacity::new(
                capacity_identity,
                usize::try_from(capacity).expect("fixture limit must fit capacity width"),
            )
            .expect("fixture capacity must be valid"),
        ),
    )
}
