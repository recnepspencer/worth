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

    let runtime = runtime_with_test_schema();
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

    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);
    let native = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
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
    let region_keys = wire
        .manifest
        .roots
        .iter()
        .flat_map(|root| &root.regions)
        .chain(&wire.manifest.mirror)
        .flat_map(|leaf| [leaf.head, leaf.history])
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        region_keys.len()
            < (sibling_root.regions.len() + main_root.regions.len() + wire.manifest.mirror.len())
                * 2
    );
    assert!(
        wire.members.is_empty(),
        "embedded checkpoint placement is retired; wire.members must be empty"
    );
    for key in region_keys {
        assert_eq!(
            native
                .members()
                .iter()
                .filter(|member| member.locator() == key)
                .count(),
            1,
            "equal regions must have exactly one external member body across roots and mirror"
        );
    }
    let restored = decode_expanded_checkpoint(
        execution,
        native.bytes(),
        crate::tests::support::durability::captured_members_source(&native),
    )
    .unwrap();
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

    let mut plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    plan.checkpoint = Some(restored.checkpoint);
    let mut recovered = runtime_with_test_schema();
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
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-roundtrip");
    let native = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    assert_eq!(wire.manifest.roots[0].regions, wire.manifest.mirror);
    assert!(wire.members.is_empty());
    let leaf = &wire.manifest.roots[0].regions[0];
    assert_eq!(
        native
            .members()
            .iter()
            .filter(|member| member.locator() == leaf.head)
            .count(),
        1
    );
    assert_eq!(
        native
            .members()
            .iter()
            .filter(|member| member.locator() == leaf.history)
            .count(),
        1
    );
    assert_eq!(wire.manifest.commits.len(), 1);
    let decoded = decode_expanded_checkpoint(
        execution,
        native.bytes(),
        crate::tests::support::durability::captured_members_source(&native),
    )
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
fn current_directory_rejects_inline_members_missing_external_members_and_digest_damage() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-damage");
    let native = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
    let full_source = crate::tests::support::durability::captured_members_source(&native);

    // 1. Non-empty wire.members (retired embedded placement) is refused at wire decode.
    {
        let mut wire: checkpoint_wire::CheckpointWire<'_> =
            rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
        let crate::durability::data::RelationalCheckpointMember::New { locator, bytes } =
            &native.members()[0]
        else {
            panic!("first external capture owns its members");
        };
        wire.members
            .push((*locator, checkpoint_wire::ByteString(bytes.bytes())));
        let mut encoded = CHECKPOINT_MAGIC.to_vec();
        encoded.extend(rmp_serde::to_vec_named(&wire).unwrap());
        let err = expect_durability_failure(
            decode_checkpoint_from_source(execution, &encoded, full_source.clone()).unwrap_err(),
        );
        assert_eq!(err.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(
            err.detail
                .contains("embedded checkpoint placement is retired"),
            "unexpected detail: {}",
            err.detail
        );
    }

    // 2. Missing external member fails expansion against an empty cold source.
    assert!(decode_expanded_checkpoint(
        execution,
        native.bytes(),
        crate::tests::support::durability::empty_cold_source()
    )
    .is_err());

    // 3. Corrupted commit digest in manifest is refused.
    {
        let mut wire: checkpoint_wire::CheckpointWire<'_> =
            rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
        let mut manifest = wire.manifest.ordinary_owned_draft();
        let mut commits = manifest.commits.iter().cloned().collect::<Vec<_>>();
        commits[0].payload_digest[0] ^= 1;
        manifest.commits = crate::durability::data::CheckpointCommitIndex::from_ordinary(commits);
        wire.manifest = manifest.seal_ordinary();
        let mut bytes = CHECKPOINT_MAGIC.to_vec();
        bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
        assert!(decode_expanded_checkpoint(execution, &bytes, full_source).is_err());
    }
}

#[test]
fn native_capture_sections_count_current_directory_values() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "manifest-sections");
    let native = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    assert!(wire.members.is_empty());
    let sections = native.captured_sections().unwrap();
    assert_eq!(sections.total, native.bytes().len());
    assert_eq!(sections.envelopes, 0);
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
    let repeated = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
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
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "retired-checkpoint");
    let native = crate::tests::support::durability::capture_checkpoint_with_source(
        execution,
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap()
    .0;
    for prefix in [
        b"WORTH-REL-CHK6\n".as_slice(),
        b"WORTH-REL-CHK17\n".as_slice(),
        b"WORTH-REL-CHK18\n".as_slice(),
        b"WORTH-REL-CHK22\n".as_slice(),
        b"WORTH-REL-CHK20\n".as_slice(),
        b"WORTH-REL-CHK20.1\n".as_slice(),
        b"WORTH-REL-CHK21\n".as_slice(),
    ] {
        let mut old = prefix.to_vec();
        old.extend_from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap());
        let stop = decode_checkpoint_from_source(
            execution,
            &old,
            crate::tests::support::durability::empty_cold_source(),
        )
        .unwrap_err();
        assert_eq!(
            expect_durability_failure(stop).class,
            RecoveryFailureClass::UnsupportedLegacySemantics
        );
    }
    for retired in [
        "partition_alias_format",
        "branch_root_partition_aliases_v2",
        "checkpoint_protocol_version",
    ] {
        let mut bytes = CHECKPOINT_MAGIC.to_vec();
        bytes.extend(
            rmp_serde::to_vec_named(&std::collections::BTreeMap::from([(retired, 8_u16)])).unwrap(),
        );
        let stop = decode_checkpoint_from_source(
            execution,
            &bytes,
            crate::tests::support::durability::empty_cold_source(),
        )
        .unwrap_err();
        assert_eq!(
            expect_durability_failure(stop).class,
            RecoveryFailureClass::CorruptCheckpoint
        );
    }
}
