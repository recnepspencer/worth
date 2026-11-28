use worth_harness::facade::ScenarioPlan;

use crate::facade::{BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};
use crate::source::{SourceDeclaration, SourceDeclarationIdentity};

fn mixed_source_declaration(
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

pub(crate) fn mixed_source_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let (scenario, commit) = NamedTruth::scenario_on_branch("analysis", "alice");
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_source_declaration(mixed_source_declaration(
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

pub(crate) fn mixed_policy_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let (truth, _commit) = ScenarioRelationalTruth::named("alice", false);
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_relational_truth(truth),
    )
    .declare_input("policy")
    .declare_observation("policy")
    .compile()
}
