use std::sync::Arc;

use crate::error::{BridgeStreamError, BridgeStreamErrorKind};
use crate::facade::RuntimeBridge;
use crate::routing::canonicalization::digest_string;
use crate::routing::BridgeRouteResult;

use super::counters::StreamProtocolCounters;
use super::declaration::diagnostics_policy_class_label;
use super::protocol::AdmittedConsumerContract;
use super::window::{PlannedChangeStreamWindow, StreamWindowIdentity};
use super::{StreamConsumerShape, StreamDeliveryIntent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamWindowDeliverySummary {
    stream_window_identity: StreamWindowIdentity,
    consumer_contract_identity: super::protocol::ConsumerContractIdentity,
    stream_digest: Arc<str>,
    window_digest: Arc<str>,
    consumer_contract_digest: Arc<str>,
    diagnostics_digest: Arc<str>,
    delivered_member_count: usize,
    delivered_route_count: usize,
    delivered_target_count: usize,
    counters: StreamProtocolCounters,
}

impl StreamWindowDeliverySummary {
    pub(crate) fn new(
        window: &PlannedChangeStreamWindow,
        delivered_route_count: usize,
        delivered_target_count: usize,
    ) -> Self {
        Self {
            stream_window_identity: window.stream_window_identity().clone(),
            consumer_contract_identity: window.consumer_contract_identity().clone(),
            stream_digest: Arc::from(window.member_set_digest()),
            window_digest: Arc::from(window.digest()),
            consumer_contract_digest: Arc::from(window.consumer_contract_identity().as_str()),
            diagnostics_digest: digest_string(
                "stream-diagnostics-policy",
                diagnostics_policy_class_label(window.diagnostics_policy_class()),
            ),
            delivered_member_count: window.members().len(),
            delivered_route_count,
            delivered_target_count,
            counters: window.counters().clone(),
        }
    }

    pub fn stream_window_identity(&self) -> &StreamWindowIdentity {
        &self.stream_window_identity
    }

    pub fn consumer_contract_identity(&self) -> &super::protocol::ConsumerContractIdentity {
        &self.consumer_contract_identity
    }

    pub fn delivered_member_count(&self) -> usize {
        self.delivered_member_count
    }

    pub fn stream_digest(&self) -> &str {
        self.stream_digest.as_ref()
    }

    pub fn window_digest(&self) -> &str {
        self.window_digest.as_ref()
    }

    pub fn consumer_contract_digest(&self) -> &str {
        self.consumer_contract_digest.as_ref()
    }

    pub fn diagnostics_digest(&self) -> &str {
        self.diagnostics_digest.as_ref()
    }

    pub fn delivered_route_count(&self) -> usize {
        self.delivered_route_count
    }

    pub fn delivered_target_count(&self) -> usize {
        self.delivered_target_count
    }

    pub fn counters(&self) -> &StreamProtocolCounters {
        &self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamWindowDeliveryResult {
    summary: StreamWindowDeliverySummary,
    route_results: Arc<[BridgeRouteResult]>,
}

impl StreamWindowDeliveryResult {
    pub(crate) fn new(
        summary: StreamWindowDeliverySummary,
        route_results: Vec<BridgeRouteResult>,
    ) -> Self {
        Self {
            summary,
            route_results: route_results.into(),
        }
    }

    pub fn summary(&self) -> &StreamWindowDeliverySummary {
        &self.summary
    }

    pub fn route_results(&self) -> &[BridgeRouteResult] {
        &self.route_results
    }
}

pub(crate) fn deliver_change_stream_window(
    runtime: &RuntimeBridge,
    contract: &AdmittedConsumerContract,
    window: &PlannedChangeStreamWindow,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StreamWindowDeliveryResult, BridgeStreamError> {
    if contract.consumer_shape() != StreamConsumerShape::RoutingConsumer {
        return Err(BridgeStreamError::new(
            BridgeStreamErrorKind::UnsupportedConsumerShape,
            "Only routing-consumer stream windows are executable in this milestone slice.",
        ));
    }
    if contract.admitted_delivery_intent() != StreamDeliveryIntent::RouteInvalidations {
        return Err(BridgeStreamError::new(
            BridgeStreamErrorKind::StreamDeliveryRejected,
            "The admitted consumer contract does not allow routing delivery for this stream path.",
        ));
    }

    if contract.consumer_contract_identity() != window.consumer_contract_identity() {
        return Err(BridgeStreamError::new(
            BridgeStreamErrorKind::StreamDeliveryRejected,
            "The admitted consumer contract did not match the planned stream window contract identity.",
        ));
    }

    let lowered_change_set = window.lowered_change_set().ok_or_else(|| {
        BridgeStreamError::new(
            BridgeStreamErrorKind::StreamDeliveryRejected,
            "The planned stream window was not lowered into an admitted delivery batch before execution.",
        )
    })?;
    let planned_routes = lowered_change_set.planned_routes().ok_or_else(|| {
        BridgeStreamError::new(
            BridgeStreamErrorKind::StreamDeliveryRejected,
            "The lowered stream batch did not carry routing delivery work for a routing consumer.",
        )
    })?;

    let route_results = planned_routes
        .iter()
        .cloned()
        .map(|route| {
            runtime
                .deliver_invalidation(route, execution)
                .map_err(|error| {
                    BridgeStreamError::new(
                        BridgeStreamErrorKind::StreamDeliveryRejected,
                        format!(
                            "Failed to deliver lowered stream work for stream window `{}`: {error}",
                            window.stream_window_identity().as_str()
                        ),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let delivered_target_count = route_results
        .iter()
        .map(|result| result.result_summary().delivered_target_count())
        .sum();
    let summary =
        StreamWindowDeliverySummary::new(window, route_results.len(), delivered_target_count);

    Ok(StreamWindowDeliveryResult::new(summary, route_results))
}
