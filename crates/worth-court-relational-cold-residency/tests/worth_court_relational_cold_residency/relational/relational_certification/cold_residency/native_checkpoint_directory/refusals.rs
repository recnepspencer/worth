//! Damage and obsolete native grammars refuse through the public receiver.
use super::*;

#[test]
fn current_directory_rejects_damaged_duplicate_and_missing_members() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "manifest-damage");
    let native = capture(execution, &runtime);
    let payload = native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let directory = read_native(&native);
    let head = directory.roots[0].regions[0].head;
    let head_bytes = member_bytes(&native, head).unwrap();
    let commit_member = native
        .members()
        .iter()
        .rev()
        .find_map(|member| match member {
            worth_relational::facade::durability::RelationalCheckpointMember::New {
                locator,
                bytes,
            } if bytes.starts_with(b"WRTHRCCA") => Some((*locator, bytes.bytes().to_vec())),
            _ => None,
        })
        .unwrap();
    let base_source = worth_testkit_relational::cold_recovery::members_source(&native);

    // 1. Non-empty inline `members` array is rejected as retired embedded placement.
    let inline_entry = array(&[
        &rmp_serde::to_vec_named(&head).unwrap(),
        &rmp_serde::to_vec(&super::super::checkpoint_wire_fixture::Bytes(head_bytes)).unwrap(),
    ]);
    let inline_bytes = replace_field(payload, "members", &array(&[&inline_entry]));
    let inline_stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(
                [CHECKPOINT_MAGIC, &inline_bytes].concat(),
            ),
            base_source.clone(),
        )
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(inline_stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    // 2. Damaged external commit member digest fails at restore readmission.
    let mut bad_commit = commit_member.1.clone();
    *bad_commit.last_mut().unwrap() ^= 1;
    let damaged_commit_source = base_source.with_published_members(&[
        worth_relational::facade::durability::RelationalCheckpointMember::New {
            locator: commit_member.0,
            bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(
                std::sync::Arc::new(bad_commit.into_boxed_slice()),
            ),
        },
    ]);
    let damaged_commit_stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(execution, &native, damaged_commit_source)
        .unwrap_err();
    let damaged_commit_err = expect_durability_failure(damaged_commit_stop);
    assert_eq!(
        damaged_commit_err.class,
        RecoveryFailureClass::ColdStorageUnavailable
    );
    assert!(
        matches!(
            damaged_commit_err.storage_unavailable,
            Some(worth_relational::facade::storage::RelationalStorageUnavailable {
                cause: worth_relational::facade::storage::RelationalStorageUnavailableCause::IntegrityMismatch,
                ..
            })
        ),
        "damaged external commit member must fail digest verification: {damaged_commit_err:?}"
    );

    // 3. Damaged external region head member fails digest verification on first demand.
    let mut bad_head = head_bytes.to_vec();
    *bad_head.last_mut().unwrap() ^= 1;
    let damaged_head_source = base_source.with_published_members(&[
        worth_relational::facade::durability::RelationalCheckpointMember::New {
            locator: head,
            bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(
                std::sync::Arc::new(bad_head.into_boxed_slice()),
            ),
        },
    ]);
    let mut cold = memory_runtime();
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &native, damaged_head_source)
        .expect("cold restore does not eagerly read unconsulted head member");
    let snapshot = snapshot_for_branch(&cold, cold.main_branch_identity().branch_id());
    let demand_err = cold
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap_err();
    assert!(
        matches!(
            demand_err,
            worth_relational::facade::transactions::RelationalExecutionDenialCause::StorageUnavailable(
                worth_relational::facade::storage::RelationalStorageUnavailable {
                    cause: worth_relational::facade::storage::RelationalStorageUnavailableCause::IntegrityMismatch
                        | worth_relational::facade::storage::RelationalStorageUnavailableCause::Damaged,
                    ..
                }
            )
        ),
        "damaged external head member must refuse on first demand: {demand_err:?}"
    );

    // 4. Duplicate member in the current CHK24 canonical_members inventory is rejected.
    let canonical = rows(field(manifest(&native), "canonical_members"));
    assert!(!canonical.is_empty());
    let duplicated = array(&[canonical[0], canonical[0]]);
    let dup_manifest = replace_field(manifest(&native), "canonical_members", &duplicated);
    let dup_payload = replace_field(payload, "manifest", &dup_manifest);
    let dup_stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(
                [CHECKPOINT_MAGIC, &dup_payload].concat(),
            ),
            base_source.clone(),
        )
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(dup_stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    // 5. Missing restore-required external commit member in cold source is rejected.
    let remaining: Vec<_> = native
        .members()
        .iter()
        .filter(|member| member.locator() != commit_member.0)
        .cloned()
        .collect();
    let missing_stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &native,
            base_source.with_selected_members(&remaining),
        )
        .unwrap_err();
    let missing_err = expect_durability_failure(missing_stop);
    assert_eq!(
        missing_err.class,
        RecoveryFailureClass::ColdStorageUnavailable
    );
    assert!(
        matches!(
            missing_err.storage_unavailable,
            Some(worth_relational::facade::storage::RelationalStorageUnavailable {
                cause: worth_relational::facade::storage::RelationalStorageUnavailableCause::MissingMember,
                ..
            })
        ),
        "missing external commit member must refuse with MissingMember: {missing_err:?}"
    );

    // 6. Tampered commit-index tail digest disagrees with inline head entry.
    let commits = field(manifest(&native), "commits");
    let tail = field(commits, "tail");
    let mut digest: [u8; 32] = rmp_serde::from_slice(field(tail, "payload_digest")).unwrap();
    digest[0] ^= 1;
    let changed_tail = replace_field(tail, "payload_digest", &rmp_serde::to_vec(&digest).unwrap());
    let changed_commits = replace_field(commits, "tail", &changed_tail);
    let changed_manifest = replace_field(manifest(&native), "commits", &changed_commits);
    let tampered_payload = replace_field(payload, "manifest", &changed_manifest);
    let tail_stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(
                [CHECKPOINT_MAGIC, &tampered_payload].concat(),
            ),
            base_source,
        )
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(tail_stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}

#[test]
fn chk6_and_retired_inner_partition_grammars_are_refused() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "retired-checkpoint");
    let native = capture(execution, &runtime);
    for prefix in [
        b"WORTH-REL-CHK6\n".as_slice(),
        b"WORTH-REL-CHK17\n".as_slice(),
        b"WORTH-REL-CHK18\n".as_slice(),
        b"WORTH-REL-CHK22\n".as_slice(),
        b"WORTH-REL-CHK23\n".as_slice(),
        b"WORTH-REL-CHK20\n".as_slice(),
        b"WORTH-REL-CHK20.1\n".as_slice(),
        b"WORTH-REL-CHK21\n".as_slice(),
    ] {
        let mut old = prefix.to_vec();
        old.extend_from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap());
        let stop = memory_runtime()
            .durability_recovery()
            .restore_native_checkpoint(
                execution,
                &RelationalNativeCheckpoint::from_untrusted_bytes(old),
                worth_testkit_relational::cold_recovery::members_source(&native),
            )
            .unwrap_err();
        assert!(matches!(
            expect_durability_failure(stop).class,
            RecoveryFailureClass::UnsupportedCheckpointVersion { supported: 24, .. }
        ));
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
        let stop = memory_runtime()
            .durability_recovery()
            .restore_native_checkpoint(
                execution,
                &RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
                worth_testkit_relational::cold_recovery::members_source(&native),
            )
            .unwrap_err();
        assert_eq!(
            expect_durability_failure(stop).class,
            RecoveryFailureClass::CorruptCheckpoint
        );
    }
}
