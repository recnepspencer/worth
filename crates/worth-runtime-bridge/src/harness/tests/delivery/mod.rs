use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeDeliveryErrorKind,
    BridgePreparationMode, BridgeRouteRequest,
};
use crate::routing::BridgeParallelAdmissionClass;

use super::support::build_runtime;
use crate::harness::fixtures::RecordingSignalBridgeSink;
mod bulk;
mod snapshot;
mod snapshot_slice_diagnostics;
