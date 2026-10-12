use super::*;
use crate::error::{BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeExecutionDenial};
#[test]
fn truth_materialization_keeps_native_execution_stops() {
    for cause in [
        BridgeExecutionDenial::Cancelled,
        BridgeExecutionDenial::DeadlineElapsed,
        BridgeExecutionDenial::WorkCeiling,
    ] {
        let source = crate::adapter::RelationalBridgeSourceError::execution_denied(cause);
        let error = BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(source),
            "source denied its snapshot",
        );
        assert_eq!(
            truth_materialization_denial(&error),
            BridgeExecutionBasisDenialKind::ExecutionDenied(cause)
        );
        let read = crate::snapshot::BridgeSnapshotReadError::execution_denied(cause);
        let error = BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::SnapshotReadFailure(read),
            "read stopped",
        );
        assert_eq!(
            truth_materialization_denial(&error),
            BridgeExecutionBasisDenialKind::ExecutionDenied(cause)
        );
    }
    let error = BridgeDeliveryError::new(
        BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
        "ordinary mismatch",
    );
    assert_eq!(
        truth_materialization_denial(&error),
        BridgeExecutionBasisDenialKind::TruthMaterializationFailed
    );
}
