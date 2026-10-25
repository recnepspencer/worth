use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::mvcc::WorkerIntentBatch;
use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
use crate::history::data::BranchId;
use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::*;

#[test]
fn member_directory_preserves_divergent_sibling_roots_and_pinned_reader() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "partition-alias-seed");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("shared-second-partition").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId(41),
                    kind_id: KindId(1),
                    client_keys: vec![crate::symbols::data::ClientKey::raw("shared-secondary")],
                    field_patches: vec![single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        "shared-secondary",
                    )],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let second = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &second);
    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&runtime, &main);
    let sibling = create_branch_from_main(&runtime, "partition-alias-sibling");
    let on_sibling =
        create_entity_outcome_on_branch(execution, &runtime, "sibling-only", sibling.clone());
    release_test_commit_snapshot(&runtime, &on_sibling);
    let on_main = create_entity_outcome(execution, &runtime, "main-only");
    release_test_commit_snapshot(&runtime, &on_main);

    let checkpoint = crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let wire: super::checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    let sibling_root = wire
        .manifest
        .roots
        .iter()
        .find(|root| root.commit_id == on_sibling.commit.commit_id)
        .unwrap();
    let main_root = wire
        .manifest
        .roots
        .iter()
        .find(|root| root.commit_id == on_main.commit.commit_id)
        .unwrap();
    let sibling_shared = sibling_root
        .regions
        .iter()
        .find(|leaf| leaf.partition_id == PartitionId(41))
        .unwrap();
    let main_shared = main_root
        .regions
        .iter()
        .find(|leaf| leaf.partition_id == PartitionId(41))
        .unwrap();
    assert_eq!(sibling_shared, main_shared);
    assert_ne!(sibling_root.regions[0].head, main_root.regions[0].head);
    assert!(
        wire.members.len()
            < (sibling_root.regions.len() + main_root.regions.len() + wire.manifest.mirror.len())
                * 2
    );
    let restored = decode_checkpoint(execution, native.bytes()).unwrap();
    assert_eq!(restored.checkpoint.branch_roots, checkpoint.branch_roots);
    assert_eq!(
        restored.checkpoint.partition_images,
        checkpoint.partition_images
    );
    assert_eq!(
        runtime
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );

    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint = Some(restored.checkpoint);
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for branch in [main, sibling] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            3
        );
    }
}

#[test]
fn current_manifest_deduplicates_equal_region_members_and_round_trips() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-roundtrip");
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    assert_eq!(wire.manifest.roots[0].regions, wire.manifest.mirror);
    let leaf = &wire.manifest.roots[0].regions[0];
    assert_eq!(
        wire.members
            .iter()
            .filter(|(key, _)| *key == leaf.head)
            .count(),
        1
    );
    assert_eq!(
        wire.members
            .iter()
            .filter(|(key, _)| *key == leaf.history)
            .count(),
        1
    );
    assert_eq!(wire.manifest.commits.len(), 1);
    let decoded = decode_checkpoint(execution, native.bytes())
        .unwrap()
        .checkpoint;
    assert_eq!(decoded.envelopes.len(), 1);
    assert_eq!(
        decoded.branch_roots[0].partition_images,
        decoded.partition_images
    );
    assert!(decoded.envelopes[0].derived_index_artifacts.is_empty());
}

#[test]
fn current_directory_rejects_damaged_duplicate_and_missing_members() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-damage");
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    for alteration in 0..4 {
        let mut wire: checkpoint_wire::CheckpointWire<'_> =
            rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
        match alteration {
            0 => wire.members[0].0.sha256[0] ^= 1,
            1 => wire.members.push((
                wire.members[0].0,
                checkpoint_wire::ByteString(wire.members[0].1 .0),
            )),
            2 => {
                let head = wire.manifest.roots[0].regions[0].head;
                wire.members.retain(|(key, _)| *key != head);
            }
            3 => wire.manifest.commits[0].payload_digest[0] ^= 1,
            _ => unreachable!(),
        }
        let mut bytes = CHECKPOINT_MAGIC.to_vec();
        bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
        assert!(
            decode_checkpoint(execution, &bytes).is_err(),
            "alteration {alteration}"
        );
    }
}

#[test]
fn native_capture_sections_count_current_directory_values() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-sections");
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    let sections = native.captured_sections().unwrap();
    assert_eq!(sections.total, native.bytes().len());
    assert_eq!(
        sections.envelopes,
        rmp_serde::to_vec_named(&wire.envelopes).unwrap().len()
    );
    assert_eq!(
        sections.branch_roots,
        rmp_serde::to_vec_named(&wire.manifest.roots).unwrap().len()
    );
    assert_eq!(
        sections.branch_cells,
        rmp_serde::to_vec_named(&wire.manifest.branch_cells)
            .unwrap()
            .len()
    );
    assert_eq!(
        sections.partition_mirror,
        rmp_serde::to_vec_named(&wire.manifest.mirror)
            .unwrap()
            .len()
    );
    assert_eq!(
        sections.derived_indexes,
        rmp_serde::to_vec_named(&wire.manifest.index_definitions)
            .unwrap()
            .len()
    );
    assert_eq!(
        sections.total,
        sections.envelopes
            + sections.branch_roots
            + sections.branch_cells
            + sections.partition_mirror
            + sections.derived_indexes
            + sections.framing_and_metadata
    );
    let repeated = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(native, repeated);
    assert_eq!(native.clone().captured_sections(), Some(sections));
    assert!(
        crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes(native.bytes())
            .captured_sections()
            .is_none()
    );
}

#[test]
fn chk6_and_retired_inner_partition_grammars_are_refused() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "retired-checkpoint");
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let mut old = b"WORTH-REL-CHK6\n".to_vec();
    old.extend_from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap());
    let stop = decode_checkpoint(execution, &old).unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        RecoveryFailureClass::UnsupportedLegacySemantics
    );
    for retired in [
        "partition_alias_format",
        "branch_root_partition_aliases_v2",
        "checkpoint_protocol_version",
    ] {
        let mut bytes = CHECKPOINT_MAGIC.to_vec();
        bytes.extend(
            rmp_serde::to_vec_named(&std::collections::BTreeMap::from([(retired, 8_u16)])).unwrap(),
        );
        let stop = decode_checkpoint(execution, &bytes).unwrap_err();
        assert_eq!(
            expect_durability_failure(stop).class,
            RecoveryFailureClass::CorruptCheckpoint
        );
    }
}
