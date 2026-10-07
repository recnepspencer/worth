use super::{
    registered_source, registered_structural, StaticSink, StaticSource, StaticSourceAdapter,
    StaticWritebackAuthority,
};
use crate::adapter::{BridgeSourceAdapter, TruthWritebackAuthority};
use crate::builder::RuntimeBridgeBuilder;
use crate::facade::RuntimeBridge;
use crate::mapping::{
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector,
    SignalInvalidationScope, TruthPatchScope,
};
use crate::policy::BridgeRuntimePolicy;
use crate::snapshot::{BridgeTruthViewSelector, SnapshotReadContract};
use crate::source::BridgeSourceCapability;
use crate::structural::{StructuralFingerprintFamily, StructuralTruthViewBasis};
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};

pub(in crate::facade::tests) fn runtime(policy: BridgeRuntimePolicy) -> RuntimeBridge {
    runtime_with_source_adapter(policy, StaticSourceAdapter)
}

pub(in crate::facade::tests) fn runtime_with_writeback_authority(
    policy: BridgeRuntimePolicy,
) -> RuntimeBridge {
    runtime_with_custom_writeback_authority(policy, StaticWritebackAuthority)
}

pub(in crate::facade::tests) fn runtime_with_custom_writeback_authority<A>(
    policy: BridgeRuntimePolicy,
    writeback_authority: A,
) -> RuntimeBridge
where
    A: TruthWritebackAuthority,
{
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .with_writeback_authority(writeback_authority)
        .register_source(analysis_snapshot_source())
        .register_source(analysis_history_source())
        .register_structural(analysis_structural_registration())
        .register_mapping(native_profile_mapping_registration())
        .build()
        .expect("runtime should build for writeback tests")
}

pub(in crate::facade::tests) fn runtime_with_source_adapter<A>(
    policy: BridgeRuntimePolicy,
    source_adapter: A,
) -> RuntimeBridge
where
    A: BridgeSourceAdapter,
{
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_source_adapter(source_adapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_source(analysis_snapshot_source())
        .register_source(analysis_history_source())
        .register_structural(analysis_structural_registration())
        .register_mapping(native_profile_mapping_registration())
        .build()
        .expect("runtime should build for policy-resolution tests")
}

fn analysis_snapshot_source() -> crate::source::SourceDeclaration {
    registered_source(
        "source:analysis-snapshot",
        BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ],
    )
}

fn analysis_history_source() -> crate::source::SourceDeclaration {
    registered_source(
        "source:analysis-history",
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        ),
        vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ],
    )
}

fn analysis_structural_registration() -> crate::structural::StructuralIdentityDeclaration {
    registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    )
}

pub(in crate::facade::tests) fn native_profile_mapping_registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("mapping"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("entity-1"),
            AspectKey::new("profile").expect("valid native aspect key"),
            FieldKey::new("name".to_owned()).expect("valid native field key"),
        ),
        SnapshotReadContract::scalar(
            AspectKey::new("profile").expect("valid native aspect key"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal:profile"),
        CoarseRoutingMode::Direct,
    )
}
