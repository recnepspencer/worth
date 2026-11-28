use serde_json::json;
use worth_harness::facade::{SnapshotPayload as HarnessSnapshotCaptureValue, StructuredValue};

pub(in crate::harness::adapter) fn historical_terminal_snapshot_capture_value(
    record: &crate::facade::BridgeCanonicalHistoricalEvaluationRecord,
) -> HarnessSnapshotCaptureValue {
    HarnessSnapshotCaptureValue::Structured(StructuredValue::Json(json!({
        "snapshot_identity": record.decision_log().snapshot_identity().canonical_label(),
        "record_identity": record.record_identity().as_str(),
    })))
}

pub(in crate::harness::adapter) fn route_terminal_snapshot_capture_value(
    record: &crate::diagnostics::BridgeRouteRecord,
) -> HarnessSnapshotCaptureValue {
    HarnessSnapshotCaptureValue::Structured(StructuredValue::Json(json!({
        "snapshot_identity": record.source_snapshot().canonical_label(),
        "read_count": record.counters().snapshot_read_count(),
    })))
}

pub(in crate::harness::adapter) fn empty_bridge_terminal_snapshot_capture_value(
) -> HarnessSnapshotCaptureValue {
    HarnessSnapshotCaptureValue::Structured(StructuredValue::Json(json!({
        "snapshot_identity": null,
        "read_count": 0,
    })))
}
