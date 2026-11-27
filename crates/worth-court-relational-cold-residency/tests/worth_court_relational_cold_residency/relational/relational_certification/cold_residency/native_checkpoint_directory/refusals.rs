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
    let base_source = worth_testkit_relational::cold_recovery::members_source(&native);
    for alteration in 0..4 {
        let mut amended = payload.to_vec();
        let mut source = base_source.clone();
        match alteration {
            0 => {
                // Non-empty inline member with damaged locator digest is rejected.
                let mut locator = head;
                locator.sha256[0] ^= 1;
                let entry = array(&[
                    &rmp_serde::to_vec_named(&locator).unwrap(),
                    &rmp_serde::to_vec(&super::super::checkpoint_wire_fixture::Bytes(head_bytes))
                        .unwrap(),
                ]);
                amended = replace_field(&amended, "members", &array(&[&entry]));
            }
            1 => {
                // Duplicate inline member is rejected.
                let entry = array(&[
                    &rmp_serde::to_vec_named(&head).unwrap(),
                    &rmp_serde::to_vec(&super::super::checkpoint_wire_fixture::Bytes(head_bytes))
                        .unwrap(),
                ]);
                amended = replace_field(&amended, "members", &array(&[&entry, &entry]));
            }
            2 => {
                // Missing restore-required external commit member in cold source is rejected.
                let commit_member = native
                    .members()
                    .iter()
                    .rev()
                    .find_map(|member| match member {
                        worth_relational::facade::durability::RelationalCheckpointMember::New {
                            locator,
                            bytes,
                        } if bytes.starts_with(b"WRTHRCCA") => Some(*locator),
                        _ => None,
                    })
                    .unwrap();
                let remaining: Vec<_> = native
                    .members()
                    .iter()
                    .filter(|member| member.locator() != commit_member)
                    .cloned()
                    .collect();
                source = base_source.with_selected_members(&remaining);
            }
            3 => {
                // Tampered commit-index tail digest disagrees with inline head entry.
                let commits = field(manifest(&native), "commits");
                let tail = field(commits, "tail");
                let mut digest: [u8; 32] =
                    rmp_serde::from_slice(field(tail, "payload_digest")).unwrap();
                digest[0] ^= 1;
                let changed_tail =
                    replace_field(tail, "payload_digest", &rmp_serde::to_vec(&digest).unwrap());
                let changed_commits = replace_field(commits, "tail", &changed_tail);
                let changed_manifest =
                    replace_field(manifest(&native), "commits", &changed_commits);
                amended = replace_field(payload, "manifest", &changed_manifest);
            }
            _ => unreachable!(),
        }
        let untrusted =
            RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &amended].concat());
        let result = memory_runtime()
            .durability_recovery()
            .restore_native_checkpoint(execution, &untrusted, source);
        assert!(result.is_err(), "alteration {alteration}");
    }
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
