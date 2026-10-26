//! A capture cannot replace canonical adjacency authority by rehashing leaves.
use super::*;
use crate::durability::data::*;
use crate::durability::medium::*;
use crate::identity::data::PartitionId;
use crate::tests::support::*;
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn forged_cross_region_kind_commitment_is_refused_before_member_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let source = create_entity_in_partition(&execution, &runtime, "source", PartitionId(1));
    let target = create_entity_in_partition(&execution, &runtime, "target", PartitionId(2));
    create_relation_in_partition(
        &execution,
        &runtime,
        source,
        target,
        "cross-region",
        PartitionId(3),
    );
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let mut wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    // Keep every copy of this leaf consistent; only canonical root authority
    // can reject the forgery. No member is needed to perform that check.
    for leaf in wire
        .manifest
        .roots
        .iter_mut()
        .flat_map(|root| &mut root.regions)
        .chain(&mut wire.manifest.mirror)
    {
        if leaf.partition_id == PartitionId(1) {
            leaf.kind_authority.head[0] ^= 1;
        }
    }
    let mut bytes = CHECKPOINT_MAGIC.to_vec();
    bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
    let source = Arc::new(member_source::InlineCheckpointSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                _ => panic!("first capture owns its members"),
            })
            .collect(),
        envelopes: attached_test_medium(&runtime)
            .offered()
            .iter()
            .map(|commit| (commit.position(), Arc::<[u8]>::from(commit.as_bytes())))
            .collect::<BTreeMap<_, _>>(),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
        )),
        Vec::new(),
        Vec::new(),
        source,
    );
    let mut receiving = durable_runtime_with_test_schema();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            &execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let failure = receiving
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap_err();
    assert!(matches!(failure, RelationalRecoveryStop::Durability(error)
        if error.class == RecoveryFailureClass::CorruptCheckpoint && error.detail.contains("StorageContentMismatch")));
}
