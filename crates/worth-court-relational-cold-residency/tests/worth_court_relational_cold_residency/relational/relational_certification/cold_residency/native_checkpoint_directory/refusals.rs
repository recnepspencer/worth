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
    for alteration in 0..4 {
        let mut body = members(&native);
        let changed_member;
        let mut amended = payload.to_vec();
        match alteration {
            0 => {
                let [transport, bytes]: [&[u8]; 2] = rows(body[0]).try_into().unwrap();
                let mut locator: RelationalDurableMemberLocator =
                    rmp_serde::from_slice(transport).unwrap();
                locator.sha256[0] ^= 1;
                changed_member = array(&[&rmp_serde::to_vec_named(&locator).unwrap(), bytes]);
                body[0] = &changed_member;
            }
            1 => body.push(body[0]),
            2 => body.retain(|member| key(member) != directory.roots[0].regions[0].head),
            3 => {
                let mut commits = rows(field(manifest(&native), "commits"));
                let mut digest: [u8; 32] =
                    rmp_serde::from_slice(field(commits[0], "payload_digest")).unwrap();
                digest[0] ^= 1;
                let changed = replace_field(
                    commits[0],
                    "payload_digest",
                    &rmp_serde::to_vec(&digest).unwrap(),
                );
                commits[0] = &changed;
                let changed = replace_field(manifest(&native), "commits", &array(&commits));
                amended = replace_field(payload, "manifest", &changed);
            }
            _ => unreachable!(),
        }
        let amended = replace_field(&amended, "members", &array(&body));
        let untrusted =
            RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &amended].concat());
        let result = memory_runtime()
            .durability_recovery()
            .restore_native_checkpoint(
                execution,
                &untrusted,
                worth_testkit_relational::cold_recovery::members_source(&untrusted),
            );
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
