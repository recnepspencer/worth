use std::collections::{BTreeMap, BTreeSet};

use worth_harness::facade::{
    run_id, AdapterSupport, CaptureDepth, DiagnosticsLevel, ExecutionMode, ExecutionProfile,
    ExecutionRequest, HarnessAdapter, HarnessCapabilities, MutationBatch, ObservationStatus,
    RecordSchemaVersion, RunOutcome, RunRecord, RunStatus, ScenarioFixture, SnapshotRecord,
    TargetStatusRecord,
};

use crate::facade::{
    BridgeDeliveryIntent, BridgeHistoricalEvaluationExplanation, BridgeMappingContext,
    BridgeReplayMode, BridgeRouteRequest, BridgeTruthViewSelector, HistoricalEvaluationDeclaration,
    RuntimeBridgeBuilder, SnapshotReadPacket, TruthBranchIdentity, TruthCommitIdentity,
};
use crate::source::BridgeSourceCapabilitySet;

use super::super::fixtures::BridgeHarnessFixture;
use super::target_id::BridgeHarnessTargetId;
use super::types::{
    BridgeHarnessAdapter, BridgeHarnessError, BridgeHarnessMutation, BridgeHarnessSession,
    PolicyBuilderLoadOrder, SourceAdapterBehavior, SourceAdapterShape, SourceBuilderLoadOrder,
};
use merge::execute_merge_request;
use policy::execute_policy_request;
use source::execute_source_request;
use speculation::execute_speculation_request;
use stream::execute_stream_request;
use structural::execute_structural_request;
use writeback::execute_writeback_request;

impl HarnessAdapter for BridgeHarnessAdapter {
    type Runtime = BridgeHarnessSession;
    type Fixture = BridgeHarnessFixture;
    type Mutation = BridgeHarnessMutation;
    type TargetId = BridgeHarnessTargetId;
    type Error = BridgeHarnessError;

    fn adapter_name(&self) -> &'static str {
        "worth-runtime-bridge"
    }

    fn capabilities(&self) -> HarnessCapabilities {
        let mut capabilities = HarnessCapabilities::default();
        capabilities.execution_modes =
            BTreeSet::from([ExecutionMode::RuntimeDefault, ExecutionMode::Serial]);
        capabilities.diagnostics_levels = BTreeSet::from([
            DiagnosticsLevel::Operational,
            DiagnosticsLevel::Development,
            DiagnosticsLevel::Forensic,
        ]);
        capabilities.capture_depths = BTreeSet::from([
            CaptureDepth::Minimal,
            CaptureDepth::Standard,
            CaptureDepth::Rich,
        ]);
        capabilities.replay_support = AdapterSupport::Supported;
        capabilities.rich_record_kinds = BTreeSet::from([
            "bridge_route_record".to_string(),
            "bridge_replay_record".to_string(),
            "bridge_delivery_receipt".to_string(),
            "bridge_merge_record".to_string(),
            "bridge_historical_evaluation_record".to_string(),
            "bridge_source_materialization_record".to_string(),
            "bridge_speculation_record".to_string(),
            "bridge_structural_remap_record".to_string(),
            "bridge_structural_branch_comparison_record".to_string(),
        ]);
        capabilities
    }

    fn create_runtime(&self) -> Result<Self::Runtime, Self::Error> {
        Ok(BridgeHarnessSession::default())
    }

    fn prepare_runtime(
        &self,
        runtime: &mut Self::Runtime,
        profile: &ExecutionProfile,
    ) -> Result<(), Self::Error> {
        runtime_loading::prepare_runtime_profile(runtime, profile)
    }

    fn load_fixture(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
    ) -> Result<(), Self::Error> {
        runtime_loading::load_bridge_fixture(runtime, fixture)
    }

    fn apply_mutation_batch(
        &self,
        runtime: &mut Self::Runtime,
        batch: &MutationBatch<Self::Mutation>,
    ) -> Result<(), Self::Error> {
        runtime_loading::apply_bridge_mutation_batch(runtime, batch)
    }

    fn execute(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<RunRecord<Self::TargetId>, Self::Error> {
        crate::host_execution::with_declared_request(
            fixture.fixture.policy().execution(),
            |resource_request| {
                let runtime_bridge = runtime
                    .runtime
                    .as_ref()
                    .ok_or_else(|| BridgeHarnessError::new("bridge runtime not loaded"))?;
                let target = request.targets.first().ok_or_else(|| {
                    BridgeHarnessError::new("bridge execution requires one target")
                })?;
                let harness_target = harness_target_from_id(target)?;
                let mapping_context = fixture
                    .fixture
                    .lineage_context()
                    .cloned()
                    .map(|lineage_context| {
                        BridgeMappingContext::default().with_lineage_context(lineage_context)
                    })
                    .unwrap_or_default();
                let scenario_id_value = worth_harness::facade::scenario_id(&fixture.name);
                let run_id_value = run_id(&scenario_id_value, &profile.name, &request.name);

                let execution = match harness_target {
                    HarnessTarget::CommittedRoute { commit_identity } => {
                        let route = runtime_bridge
                            .plan_committed_patch_with_mapping_context(
                                BridgeRouteRequest::for_commit(commit_identity),
                                mapping_context,
                                resource_request,
                            )
                            .map_err(|error| {
                                BridgeHarnessError::new(format!("bridge planning failed: {error}"))
                            })?;
                        let result = runtime_bridge
                            .deliver_invalidation(route, resource_request)
                            .map_err(|error| {
                                BridgeHarnessError::new(format!("bridge delivery failed: {error}"))
                            })?;
                        let continuity_summary = runtime_bridge
                            .diagnostics()
                            .last_route_record()
                            .and_then(|route_record| {
                                let requests = runtime_bridge
                                    .plan_continuity_requests(&route_record)
                                    .ok()?;
                                let packet = runtime_bridge
                                    .plan_historical_lineage_packet(&requests)
                                    .ok()?;
                                let resolved =
                                    runtime_bridge.resolve_lineage_continuity(&packet).ok()?;
                                let artifact = runtime_bridge.lower_continuity_artifact(&resolved);
                                let canonical = runtime_bridge.canonicalize_continuity_record(
                                    &route_record,
                                    &requests,
                                    &artifact,
                                );
                                Some((artifact, canonical))
                            });
                        HarnessExecution::Route {
                            result,
                            continuity_summary,
                        }
                    }
                    HarnessTarget::Stream(stream_target) => HarnessExecution::Stream(
                        execute_stream_request(runtime_bridge, stream_target, resource_request)?,
                    ),
                    HarnessTarget::Source(source_target) => {
                        HarnessExecution::Source(execute_source_request(
                            runtime_bridge,
                            &fixture.fixture,
                            source_target,
                            resource_request,
                        )?)
                    }
                    HarnessTarget::Merge(merge_target) => HarnessExecution::Merge(
                        execute_merge_request(runtime_bridge, &fixture.fixture, merge_target)?,
                    ),
                    HarnessTarget::Policy(policy_target) => {
                        HarnessExecution::Policy(execute_policy_request(
                            runtime_bridge,
                            &fixture.fixture,
                            policy_target,
                            resource_request,
                        )?)
                    }
                    HarnessTarget::Speculation(speculation_target) => {
                        HarnessExecution::Speculation(execute_speculation_request(
                            runtime_bridge,
                            &fixture.fixture,
                            speculation_target,
                            resource_request,
                        )?)
                    }
                    HarnessTarget::Structural(structural_target) => {
                        HarnessExecution::Structural(execute_structural_request(
                            runtime_bridge,
                            &fixture.fixture,
                            structural_target,
                            resource_request,
                        )?)
                    }
                    HarnessTarget::Writeback(writeback_target) => {
                        HarnessExecution::Writeback(execute_writeback_request(
                            runtime,
                            runtime_bridge,
                            &fixture.fixture,
                            writeback_target,
                            resource_request,
                        )?)
                    }
                    HarnessTarget::HistoricalCommit {
                        branch_identity,
                        commit_identity,
                    } => execute_historical_request(
                        runtime_bridge,
                        HistoricalEvaluationDeclaration::new(
                            BridgeTruthViewSelector::historical_commit(
                                branch_identity,
                                commit_identity,
                            ),
                            BridgeReplayMode::Enabled,
                            fixture.fixture.policy().diagnostics_tier(),
                            BridgeDeliveryIntent::PrepareSignalEvaluation,
                        ),
                        resource_request,
                    )?,
                    HarnessTarget::BranchHead { branch_identity } => execute_historical_request(
                        runtime_bridge,
                        HistoricalEvaluationDeclaration::new(
                            BridgeTruthViewSelector::branch_head(branch_identity),
                            BridgeReplayMode::Enabled,
                            fixture.fixture.policy().diagnostics_tier(),
                            BridgeDeliveryIntent::PrepareSignalEvaluation,
                        ),
                        resource_request,
                    )?,
                };

                Ok(RunRecord {
                    schema_version: RecordSchemaVersion::V1,
                    run_id: run_id_value,
                    scenario_id: scenario_id_value,
                    adapter_name: self.adapter_name().to_string(),
                    scenario_name: fixture.name.clone(),
                    profile_name: profile.name.clone(),
                    time_marker: profile.time_marker.clone(),
                    feed_batch: request.feed_batch.clone(),
                    execution_mode: profile.execution_mode,
                    diagnostics_level: profile.diagnostics_level,
                    status: RunStatus::Succeeded,
                    outcome: RunOutcome::Completed,
                    budget_usage: None,
                    requested_targets: request.targets.clone(),
                    target_statuses: request
                        .targets
                        .iter()
                        .map(|target| TargetStatusRecord {
                            target: target.clone(),
                            status: ObservationStatus::Validated,
                            detail: None,
                        })
                        .collect(),
                    changed_targets: request.targets.clone(),
                    attachments: Vec::new(),
                    summary: terminal_report_export::execution_summary_json(&execution),
                    extensions: terminal_report_export::execution_extensions_json(
                        &execution,
                        runtime_bridge,
                    ),
                })
            },
        )
    }

    fn capture_snapshot(
        &self,
        runtime: &Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<SnapshotRecord<Self::TargetId>, Self::Error> {
        snapshot_capture::capture_bridge_snapshot_record(
            self.adapter_name(),
            runtime,
            fixture,
            request,
            profile,
        )
    }
}

mod merge;
mod policy;
mod runtime_loading;
mod snapshot_capture;
mod source;
mod speculation;
mod stream;
mod structural;
mod terminal_report_export;
mod writeback;
mod writeback_certification;
use execution::{
    execute_historical_request, harness_target_from_id, HarnessExecution, HarnessTarget,
};
mod execution;
