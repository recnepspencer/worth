use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeContinuityAuthorityBasis,
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageContext, BridgeLineageSourceError, BridgeRouteRequest, ContinuityLineageSource,
    BRIDGE_CANONICAL_BULK_PLAN_RECORD_SCHEMA_V1,
};

use super::support::{
    build_runtime, committed_patch, committed_patch_items, field_aspect_registration,
    field_slice_snapshot, registration, snapshot,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[derive(Debug, Clone, Default)]
struct ReplaySingleSuccessorLineageSource;

impl ContinuityLineageSource for ReplaySingleSuccessorLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:replay-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct ReplayDriftedSuccessorLineageSource;

impl ContinuityLineageSource for ReplayDriftedSuccessorLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:replay-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:9:2",
            )],
            vec![7],
        )
    }
}

mod bulk;
mod continuity;
mod route;
