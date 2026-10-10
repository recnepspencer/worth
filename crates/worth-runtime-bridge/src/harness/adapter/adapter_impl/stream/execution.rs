use super::{NativeStreamCommitWindow, StreamHarnessTarget};
use crate::facade::{
    AdmittedConsumerContract, BridgeRouteRequest, CanonicalStreamReplayRecord,
    ChangeStreamDeclaration, ConsumerCheckpointToken, PlannedChangeStreamWindow,
    StreamCheckpointFrontierKind, StreamCoalescingFamily, StreamCoalescingIntent,
    StreamDeliveryIntent, StreamDiagnosticsPolicyClass, StreamReplayAuditResult, StreamReplayMode,
    StreamResumeMode, StreamWindowDeliveryResult, ValidatedStreamProtocol,
};
use crate::harness::adapter::adapter_impl::BridgeHarnessError;
use crate::stream::StreamConsumerShape;

pub(in crate::harness::adapter::adapter_impl) enum StreamHarnessExecution {
    Routing {
        window: PlannedChangeStreamWindow,
        result: StreamWindowDeliveryResult,
        checkpoint: ConsumerCheckpointToken,
        replay_record: CanonicalStreamReplayRecord,
    },
    ReplayAudit {
        window: PlannedChangeStreamWindow,
        result: StreamReplayAuditResult,
    },
}

pub(in crate::harness::adapter::adapter_impl) fn execute_stream_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    target: StreamHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StreamHarnessExecution, BridgeHarnessError> {
    match target {
        StreamHarnessTarget::RoutingWindow {
            window: native_window,
        } => execute_routing_stream_window(runtime_bridge, &native_window, resource_request),
        StreamHarnessTarget::ReplayAuditWindow {
            window: native_window,
        } => execute_replay_audit_stream_window(runtime_bridge, &native_window, resource_request),
    }
}

fn execute_routing_stream_window(
    runtime_bridge: &crate::facade::RuntimeBridge,
    native_window: &NativeStreamCommitWindow,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StreamHarnessExecution, BridgeHarnessError> {
    let protocol =
        validate_stream_declaration(runtime_bridge, StreamConsumerShape::RoutingConsumer)?;
    let contract = resolve_stream_contract(runtime_bridge, &protocol)?;
    let window = plan_stream_window(runtime_bridge, &contract, native_window, execution)?;
    let result = runtime_bridge
        .deliver_change_stream_window(&contract, &window, execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge stream delivery failed: {error}"))
        })?;
    let checkpoint = runtime_bridge.publish_consumer_checkpoint(
        &contract,
        &window,
        StreamCheckpointFrontierKind::ContiguousFrontier,
    );
    let replay_record = runtime_bridge
        .canonicalize_stream_replay_record(&contract, &window, &checkpoint)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge stream replay record canonicalization failed: {error}"
            ))
        })?;
    Ok(StreamHarnessExecution::Routing {
        window,
        result,
        checkpoint,
        replay_record,
    })
}

fn execute_replay_audit_stream_window(
    runtime_bridge: &crate::facade::RuntimeBridge,
    native_window: &NativeStreamCommitWindow,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StreamHarnessExecution, BridgeHarnessError> {
    let protocol =
        validate_stream_declaration(runtime_bridge, StreamConsumerShape::ReplayAuditConsumer)?;
    let contract = resolve_stream_contract(runtime_bridge, &protocol)?;
    let window = plan_stream_window(runtime_bridge, &contract, native_window, resource_request)?;
    let result = runtime_bridge
        .deliver_replay_audit_stream_window(&contract, &window)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge stream replay-audit failed: {error}"))
        })?;
    Ok(StreamHarnessExecution::ReplayAudit { window, result })
}

fn validate_stream_declaration(
    runtime_bridge: &crate::facade::RuntimeBridge,
    consumer_shape: StreamConsumerShape,
) -> Result<ValidatedStreamProtocol, BridgeHarnessError> {
    runtime_bridge
        .validate_change_stream_declaration(stream_declaration(runtime_bridge, consumer_shape))
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge stream declaration validation failed: {error}"
            ))
        })
}

fn resolve_stream_contract(
    runtime_bridge: &crate::facade::RuntimeBridge,
    protocol: &ValidatedStreamProtocol,
) -> Result<AdmittedConsumerContract, BridgeHarnessError> {
    runtime_bridge
        .resolve_change_stream_consumer_contract(protocol)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge stream contract resolution failed: {error}"))
        })
}

fn plan_stream_window(
    runtime_bridge: &crate::facade::RuntimeBridge,
    contract: &AdmittedConsumerContract,
    native_window: &NativeStreamCommitWindow,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PlannedChangeStreamWindow, BridgeHarnessError> {
    let mut envelopes = Vec::with_capacity(native_window.commits().len());
    for commit in native_window.commits() {
        let envelope = runtime_bridge
            .ingest_committed_patch(BridgeRouteRequest::for_commit(commit.clone()), execution)
            .map_err(|error| {
                BridgeHarnessError::new(format!(
                    "bridge stream ingestion failed for `{}`: {error}",
                    commit.as_str()
                ))
            })?;
        envelopes.push(envelope);
    }
    runtime_bridge
        .plan_change_stream_window(contract, envelopes)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge stream window planning failed: {error}"))
        })
}

fn stream_declaration(
    runtime_bridge: &crate::facade::RuntimeBridge,
    consumer_shape: StreamConsumerShape,
) -> ChangeStreamDeclaration {
    let (coalescing_intent, delivery_intent) = match consumer_shape {
        StreamConsumerShape::RoutingConsumer => (
            StreamCoalescingIntent::Prefer(StreamCoalescingFamily::RoutingWindowCoalescing),
            StreamDeliveryIntent::RouteInvalidations,
        ),
        StreamConsumerShape::ReplayAuditConsumer => (
            StreamCoalescingIntent::Prefer(StreamCoalescingFamily::ReplayAuditWindowCoalescing),
            StreamDeliveryIntent::ReplayAudit,
        ),
    };
    ChangeStreamDeclaration::new(
        consumer_shape,
        StreamResumeMode::FromCheckpointOnly,
        crate::facade::StreamCheckpointPublicationMode::PublishEveryWindow,
        coalescing_intent,
        StreamReplayMode::Enabled,
        delivery_intent,
        diagnostics_policy_class(runtime_bridge.policy().diagnostics_tier()),
    )
}

fn diagnostics_policy_class(
    tier: crate::facade::BridgeDiagnosticsTier,
) -> StreamDiagnosticsPolicyClass {
    match tier {
        crate::facade::BridgeDiagnosticsTier::Minimal => StreamDiagnosticsPolicyClass::Minimal,
        crate::facade::BridgeDiagnosticsTier::Standard => StreamDiagnosticsPolicyClass::Standard,
        crate::facade::BridgeDiagnosticsTier::Exhaustive => {
            StreamDiagnosticsPolicyClass::Exhaustive
        }
    }
}
