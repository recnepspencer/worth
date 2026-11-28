use worth_harness::facade::ScenarioPlan;
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, HarnessAdapter, RunRecord};

use crate::facade::{BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth};
use crate::source::{SourceDeclaration, SourceDeclarationIdentity};

pub(super) fn source_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let (scenario, commit) = NamedTruth::scenario_on_branch("analysis", "alice");
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_source_declaration(historical_source_declaration(
                SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
                commit,
            ))
            .with_source_adapter_capabilities(BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ]))
            .with_relational_truth(scenario),
    )
    .declare_input("source")
    .declare_observation("source")
    .compile()
}

pub(super) fn historical_source_declaration(
    declaration_identity: SourceDeclarationIdentity,
    commit: crate::facade::TruthCommitIdentity,
) -> SourceDeclaration {
    SourceDeclaration::new(
        declaration_identity,
        BridgeTruthViewSelector::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("analysis"),
            commit,
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ]),
    )
}

pub(super) fn materialize_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_materialize(SourceDeclarationIdentity::admit_bridge_owned(
        "source:analysis-history",
    ))
}

pub(super) fn replay_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_replay(SourceDeclarationIdentity::admit_bridge_owned(
        "source:analysis-history",
    ))
}

pub(super) fn materialize_batch_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_materialize_batch(SourceDeclarationIdentity::admit_bridge_owned(
        "source:analysis-history",
    ))
}

pub(super) fn hostile_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_reject_unregistered(
        SourceDeclarationIdentity::admit_bridge_owned("source:hostile-missing"),
    )
}

pub(super) fn reject_open_snapshot_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_reject_open_snapshot(
        SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
    )
}

pub(super) fn reject_snapshot_drift_target() -> BridgeHarnessTargetId {
    BridgeHarnessTargetId::source_reject_snapshot_drift(
        SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
    )
}

pub(super) fn direct_host_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(name)
}

pub(super) fn wrapped_host_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-wrapped-host"))
        .with_metadata("source_adapter_shape", "wrapped")
}

pub(super) fn sources_first_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-sources-first"))
        .with_metadata("source_builder_load_order", "sources_first")
}

pub(super) fn rejecting_adapter_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-reject-open-snapshot"))
        .with_metadata("source_adapter_behavior", "reject_open_snapshot")
}

pub(super) fn drifting_adapter_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-drift-snapshot-identity"))
        .with_metadata("source_adapter_behavior", "drift_snapshot_identity")
}

pub(super) fn execute_source_run(
    profile: ExecutionProfile,
    request_name: &str,
    target: BridgeHarnessTargetId,
) -> RunRecord<BridgeHarnessTargetId> {
    let adapter = BridgeHarnessAdapter;
    let fixture = source_fixture("bridge-source-matrix");
    let mut runtime = adapter.create_runtime().expect("source harness runtime");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("source harness prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("source harness load fixture");
    adapter
        .execute(
            &mut runtime,
            &fixture,
            &ExecutionRequest::target(request_name, target),
            &profile,
        )
        .expect("source harness execution")
}
