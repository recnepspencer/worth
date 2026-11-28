use worth_harness::facade::ScenarioPlan;

use super::super::support::{
    coarse_snapshot_record, committed_patch, committed_patch_on_branch, registration, snapshot,
};
use crate::facade::{
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector,
    StructuralFingerprintEquivalenceContract, StructuralFingerprintFamily,
    StructuralFingerprintNormalizationRule, StructuralFingerprintOmissionPolicy,
    StructuralFingerprintOrderingRule, StructuralIdentityDeclaration,
    StructuralIdentityDeclarationIdentity, StructuralSchemaIdentity, StructuralTruthViewBasis,
    TruthSnapshotIdentity,
};
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::source::{SourceDeclaration, SourceDeclarationIdentity};

pub(crate) fn mixed_stream_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("stream")
    .declare_observation("stream")
    .compile()
}

fn mixed_source_declaration(declaration_identity: SourceDeclarationIdentity) -> SourceDeclaration {
    SourceDeclaration::new(
        declaration_identity,
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
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
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_source_declaration(mixed_source_declaration(
                SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
            ))
            .with_source_adapter_capabilities(BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ]))
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("source")
    .declare_observation("source")
    .compile()
}

fn mixed_structural_snapshot(
    snapshot_identity: TruthSnapshotIdentity,
    value: &str,
) -> crate::harness::fixtures::SnapshotFixture {
    let mismatch_identity = snapshot_identity.as_str().to_owned();
    crate::harness::fixtures::SnapshotFixture::new(
        snapshot_identity,
        vec![
            coarse_snapshot_record(
                "entity-1",
                "profile",
                worth_foundational::facade::AspectValue::String(value.into()),
            ),
            coarse_snapshot_record(
                "entity-2",
                "profile",
                worth_foundational::facade::AspectValue::String(value.into()),
            ),
            coarse_snapshot_record(
                "entity-3",
                "profile",
                worth_foundational::facade::AspectValue::String(
                    format!("shape-mismatch-{mismatch_identity}").into(),
                ),
            ),
        ],
    )
}

fn mixed_structural_remap_declaration(
    declaration_identity: StructuralIdentityDeclarationIdentity,
) -> StructuralIdentityDeclaration {
    StructuralIdentityDeclaration::advisory_remap(
        declaration_identity,
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        StructuralFingerprintEquivalenceContract::new(
            StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
            StructuralFingerprintFamily::TopologyFingerprint,
            "geometry-topology-v1",
            StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
            StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
            StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
        ),
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    )
}

pub(crate) fn mixed_structural_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_structural_declaration(mixed_structural_remap_declaration(
                StructuralIdentityDeclarationIdentity::admit_bridge_owned(
                    "structural:analysis-remap",
                ),
            ))
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(mixed_structural_snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            ))
            .with_snapshot(mixed_structural_snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                "bob",
            )),
    )
    .declare_input("structural")
    .declare_observation("structural")
    .compile()
}

pub(crate) fn mixed_policy_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("policy")
    .declare_observation("policy")
    .compile()
}

pub(crate) fn mixed_speculation_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("speculation")
    .declare_observation("speculation")
    .compile()
}

pub(crate) fn mixed_writeback_fixture(
    name: &str,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("writeback")
    .declare_observation("writeback")
    .compile()
}
