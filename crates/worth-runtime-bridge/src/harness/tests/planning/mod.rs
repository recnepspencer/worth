use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

use crate::facade::{
    BridgeBulkDecisionRecordKind, BridgeBulkPlanningFailureKind, BridgeBulkWorkloadRequest,
    BridgeBulkWorkloadSegment, BridgeContinuityAuthorityBasis, BridgeDiagnosticsTier,
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageContext, BridgeLineageSourceError, BridgeMappingContext,
    BridgeParallelAdmissionClass, BridgeParallelAdmissionReason, BridgeParallelLegalityClass,
    BridgeParallelLegalityReason, BridgeParallelProfitabilityClass,
    BridgeParallelProfitabilityReason, BridgePreparationMode, BridgeRouteRequest,
    BridgeRuntimePolicy, ContinuityLineageSource, FineGrainedMatchStatus, SubscriptionSliceKind,
};

use super::support::{
    build_runtime, committed_patch, committed_patch_items, field_aspect_registration,
    field_slice_snapshot, registration, snapshot, surface_widening_registration,
};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{
    BridgeHarnessFixture, InMemoryRelationalBridgeSource, RecordingSignalBridgeSink,
};

#[derive(Debug, Clone, Default)]
struct TestContinuityLineageSource;

impl ContinuityLineageSource for TestContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:test-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct TestMismatchedAuthorityLineageSource;

impl ContinuityLineageSource for TestMismatchedAuthorityLineageSource {
    fn historical_lineage(
        &self,
        _request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            BridgeContinuityAuthorityBasis::new(
                crate::truth_identity_fixtures::truth_branch_fixture("wrong-branch"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:test-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct TestUnsupportedContinuityLineageSource;

impl ContinuityLineageSource for TestUnsupportedContinuityLineageSource {
    fn historical_lineage(
        &self,
        _request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        Err(crate::facade::BridgeLineageSourceError::new(
            crate::facade::BridgeLineageSourceErrorKind::UnsupportedContinuityClass,
            "unsupported continuity class",
        ))
    }
}

#[derive(Debug, Clone, Default)]
struct TestSplitContinuityLineageSource;

impl ContinuityLineageSource for TestSplitContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned("lineage:test-split-a"),
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned("lineage:test-split-b"),
            ],
            vec![
                BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned("entity:0:4:2"),
                BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned("entity:0:5:2"),
            ],
            vec![7, 8],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct TestMergeLikeContinuityLineageSource;

impl ContinuityLineageSource for TestMergeLikeContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned("lineage:test-merge-a"),
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned("lineage:test-merge-b"),
            ],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:9:3",
            )],
            vec![7, 8],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct TestAmbiguousContinuityLineageSource;

impl ContinuityLineageSource for TestAmbiguousContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                    "lineage:test-ambiguous-a",
                ),
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                    "lineage:test-ambiguous-b",
                ),
                BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                    "lineage:test-ambiguous-c",
                ),
            ],
            vec![
                BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned("entity:0:4:2"),
                BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned("entity:0:5:2"),
            ],
            vec![7, 8, 9],
        )
    }
}

#[derive(Debug, Clone, Default)]
struct TestNoAuthoritativeSuccessorLineageSource;

impl ContinuityLineageSource for TestNoAuthoritativeSuccessorLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            Vec::new(),
            Vec::new(),
            vec![7],
        )
    }
}

#[derive(Debug, Clone)]
struct CountingContinuityLineageSource {
    call_count: Arc<AtomicUsize>,
}

impl CountingContinuityLineageSource {
    fn new() -> Self {
        Self {
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl ContinuityLineageSource for CountingContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:test-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}

mod bulk_workload;
mod continuity;
mod packet_reduction;
mod route_identity;
