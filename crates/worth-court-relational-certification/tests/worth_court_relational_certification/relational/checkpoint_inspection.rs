//! Independent authenticated-frame rewrites exercise the codec's closed schema.
use crate::messagepack_value_spans::{fields, sequence};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        CheckpointRecordIdentity, RecordAllocationClass, RecoveryFailureClass,
        RelationalDurableMemberLocator, RelationalNativeCheckpoint, RelationalRecoveryStop,
    },
    history::BranchId,
    mvcc::RelationalTransactionIntent,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::{batch_create, memory_runtime},
    maintenance::relation_laws::{request, World},
    ordinary_records::create_entity,
};

mod shared_members;

const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK23\n";

fn original(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalNativeCheckpoint, [CheckpointRecordIdentity; 2]) {
    let world = World::new(execution, 1);
    let entity = world.nodes[0];
    let relation = world.edges[0].0;
    (
        world
            .runtime
            .durability_authority()
            .native_checkpoint_with_policy(
execution,
worth_execution::ExecutionAllocationPolicy::SystemAllocation,
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
            .unwrap(),
        [
            CheckpointRecordIdentity {
                class: RecordAllocationClass::Entity,
                partition_id: entity.partition_value(),
                slot: entity.local_slot_value(),
                generation: entity.generation_value(),
            },
            CheckpointRecordIdentity {
                class: RecordAllocationClass::Relation,
                partition_id: relation.partition_value(),
                slot: relation.local_slot_value(),
                generation: relation.generation_value(),
            },
        ],
    )
}
fn map_bytes(fields: BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![0xde];
    bytes.extend(u16::try_from(fields.len()).unwrap().to_be_bytes());
    for (key, value) in fields {
        bytes.extend(rmp_serde::to_vec(&key).unwrap());
        bytes.extend(value);
    }
    bytes
}
fn map(bytes: &[u8], edit: impl FnOnce(&mut BTreeMap<String, Vec<u8>>)) -> Vec<u8> {
    let mut value = fields(bytes)
        .into_iter()
        .map(|(k, v)| (k, v.to_vec()))
        .collect();
    edit(&mut value);
    map_bytes(value)
}
/// Edit just one row while preserving every unrelated value's raw encoding.
fn rewrite_row(
    bytes: &[u8],
    index: usize,
    edit: impl FnOnce(&mut BTreeMap<String, Vec<u8>>),
) -> Vec<u8> {
    let row = sequence(bytes)[index];
    let start = row.as_ptr() as usize - bytes.as_ptr() as usize;
    let end = start + row.len();
    [
        bytes[..start].to_vec(),
        map(row, edit),
        bytes[end..].to_vec(),
    ]
    .concat()
}
fn rewritten(
    bytes: &[u8],
    allocator: bool,
    edit: impl FnOnce(&mut BTreeMap<String, Vec<u8>>),
) -> Vec<u8> {
    let envelope = fields(&bytes[CHECKPOINT_MAGIC.len()..]);
    let checkpoint = map(envelope["manifest"], |stored| {
        if allocator {
            let value = map(&stored["record_identity"], edit);
            stored.insert("record_identity".into(), value);
        } else {
            edit(stored);
        }
    });
    let payload = map(&bytes[CHECKPOINT_MAGIC.len()..], |fields| {
        fields.insert("manifest".into(), checkpoint);
    });
    [CHECKPOINT_MAGIC, payload.as_slice()].concat()
}
/// Change an encoding's integer/length width without changing its value.
fn wider(old: &[u8]) -> Option<Vec<u8>> {
    let mut result = match old[0] {
        n @ 0x00..=0x7f => {
            let mut v = vec![0xcf];
            v.extend(u64::from(n).to_be_bytes());
            return Some(v);
        }
        n @ 0x80..=0x8f => {
            let mut v = vec![0xde];
            v.extend(u16::from(n - 0x80).to_be_bytes());
            v
        }
        n @ 0x90..=0x9f => {
            let mut v = vec![0xdc];
            v.extend(u16::from(n - 0x90).to_be_bytes());
            v
        }
        n @ 0xa0..=0xbf => vec![0xd9, n - 0xa0],
        0xdc | 0xde => {
            let mut v = vec![old[0] + 1];
            v.extend(u32::from(u16::from_be_bytes(old[1..3].try_into().unwrap())).to_be_bytes());
            v.extend(&old[3..]);
            return Some(v);
        }
        0xd9 => {
            let mut v = vec![0xda];
            v.extend(u16::from(old[1]).to_be_bytes());
            v.extend(&old[2..]);
            return Some(v);
        }
        _ => return None,
    };
    result.extend(&old[1..]);
    Some(result)
}
#[test]
fn inspection_compares_every_native_model_value_as_original_bytes() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, identities) = original(&execution);
    let decoded = before.inspect(&execution).unwrap();
    for identity in identities {
        assert!(
            decoded.live_records.contains(&identity),
            "the census uses the actual committed identity: {identity:?}"
        );
    }
    let envelope = fields(&before.bytes()[CHECKPOINT_MAGIC.len()..]);
    for (key, old) in fields(envelope["manifest"]) {
        if matches!(key.as_str(), "record_identity" | "symbol_identity_frontier") {
            continue;
        }
        let equivalent = wider(old);
        let after = rewritten(before.bytes(), false, |fields| {
            fields.insert(
                key.clone(),
                equivalent.clone().unwrap_or_else(|| vec![0xc3]),
            );
        });
        match RelationalNativeCheckpoint::inspect_bytes(&execution, &after) {
            Ok(after) => assert_ne!(decoded.model, after.model, "ignored model field {key}"),
            Err(error) => assert!(
                equivalent.is_none(),
                "equivalent {key} encoding refused: {error:?}"
            ),
        }
    }
    let checkpoint = fields(envelope["manifest"]);
    for (key, old) in fields(checkpoint["record_identity"]) {
        if matches!(key.as_str(), "append_frontiers" | "generation_high_water") {
            continue;
        }
        let equivalent =
            wider(old).expect("allocator model fields have an equivalent wider encoding");
        let after = rewritten(before.bytes(), true, |fields| {
            fields.insert(key.clone(), equivalent);
        });
        let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &after).unwrap();
        assert_ne!(
            decoded.model, after.model,
            "ignored allocator model field {key}"
        );
    }
}
#[test]
fn inspection_refuses_unknown_root_allocator_and_row_fields() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, _) = original(&execution);
    for allocator in [false, true] {
        let after = rewritten(before.bytes(), allocator, |fields| {
            fields.insert("future_field".into(), vec![0xc0]);
        });
        assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err());
    }
    for key in ["append_frontiers", "generation_high_water"] {
        let after = rewritten(before.bytes(), true, |fields| {
            let mut rows: Vec<serde_json::Value> = rmp_serde::from_slice(&fields[key]).unwrap();
            assert!(!rows.is_empty());
            rows[0]["future_field"] = true.into();
            fields.insert(key.into(), rmp_serde::to_vec_named(&rows).unwrap());
        });
        assert!(
            RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err(),
            "unknown field in {key} row"
        );
    }
}

#[test]
fn restore_refuses_unknown_fields_before_publishing_any_model_state() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (original, _) = original(&execution);
    let mut control = World::empty_runtime(0);
    control
        .durability_recovery()
        .restore_native_checkpoint(&execution, &original, worth_testkit_relational::cold_recovery::members_source(&original))
        .unwrap();
    assert!(control.history().immutable_commit_count() > 0);
    for owner in [
        "manifest",
        "record_identity",
        "append_frontiers",
        "generation_high_water",
        "roots",
        "mirror",
    ] {
        let allocator = matches!(
            owner,
            "record_identity" | "append_frontiers" | "generation_high_water"
        );
        let bytes = rewritten(original.bytes(), allocator, |fields| {
            if matches!(owner, "manifest" | "record_identity") {
                fields.insert("future_field".into(), vec![0xc0]);
            } else {
                let rows = rewrite_row(&fields[owner], 0, |row| {
                    row.insert("future_field".into(), vec![0xc0]);
                });
                fields.insert(owner.into(), rows);
            }
        });
        let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes);
        let mut target = World::empty_runtime(0);
        let before = target
            .durability_authority()
            .native_checkpoint_with_policy(
&execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
            .unwrap();
        let denied = target
            .durability_recovery()
            .restore_native_checkpoint(&execution, &checkpoint, worth_testkit_relational::cold_recovery::members_source(&checkpoint))
            .unwrap_err();
        let RelationalRecoveryStop::Durability(error) = denied else {
            panic!("wrong restore refusal: {denied:?}")
        };
        assert_eq!(
            error.class,
            RecoveryFailureClass::CorruptCheckpoint,
            "{owner}"
        );
        assert!(
            error.detail.contains("unknown field") && error.detail.contains("future_field"),
            "{owner}: {error:?}"
        );
        let after = target
            .durability_authority()
            .native_checkpoint_with_policy(
&execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
            .unwrap();
        assert_eq!(
            before.bytes(),
            after.bytes(),
            "restore changed state for {owner}"
        );
    }
}

#[test]
fn the_live_census_includes_divergent_retained_branch_roots() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let mut expected = vec![create_entity(&execution, &runtime, "shared-root")];
    for branch in ["left-inspection", "right-inspection"] {
        let branch_id = BranchId(branch.into());
        let (_, source) = runtime
            .observe_fork_source(runtime.main_branch_identity().branch_id())
            .unwrap();
        runtime
            .fork_branch(&execution, branch_id.clone(), source)
            .unwrap();
        let branch = runtime.branch_identity(&branch_id).unwrap();
        let (_, basis) = runtime.observe_branch(&branch).unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                batch_create(branch_id.0.as_str()),
                execution.allocation_policy(),
            )
            .unwrap();
        let commit = transaction.commit(&execution, &runtime).unwrap();
        expected.extend(changed_entities(&commit));
        release_test_commit_snapshot(&runtime, &commit);
    }
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
&execution,
worth_execution::ExecutionAllocationPolicy::SystemAllocation,
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap();
    let inspected = checkpoint.inspect(&execution).unwrap();
    for entity in expected {
        assert!(
            inspected.live_records.contains(&CheckpointRecordIdentity {
                class: RecordAllocationClass::Entity,
                partition_id: entity.partition_value(),
                slot: entity.local_slot_value(),
                generation: entity.generation_value(),
            }),
            "missing live entity from a retained branch: {entity:?}"
        );
    }
}

#[test]
fn burned_symbol_frontier_does_not_hide_published_symbol_damage() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, _) = original(&execution);
    let inspected = before.inspect(&execution).unwrap();
    let next = inspected.symbol_identity_frontier.checked_add(7).unwrap();
    let advanced = rewritten(before.bytes(), false, |fields| {
        fields.insert(
            "symbol_identity_frontier".into(),
            rmp_serde::to_vec(&next).unwrap(),
        );
    });
    let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &advanced).unwrap();
    assert_eq!(after.symbol_identity_frontier, next);
    assert_eq!(after.model, inspected.model);
    assert_eq!(after.live_records, inspected.live_records);
    let invalid = rewritten(before.bytes(), false, |fields| {
        fields.insert(
            "symbol_identity_frontier".into(),
            rmp_serde::to_vec(&0_u32).unwrap(),
        );
    });
    assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &invalid).is_err());
    let envelope = fields(&before.bytes()[CHECKPOINT_MAGIC.len()..]);
    let symbols = sequence(fields(envelope["manifest"])["symbols"]);
    let symbol: RelationalDurableMemberLocator =
        rmp_serde::from_slice(fields(symbols[0])["member"]).unwrap();
    let member = sequence(envelope["members"])
        .into_iter()
        .map(sequence)
        .find(|member| {
            rmp_serde::from_slice::<RelationalDurableMemberLocator>(member[0]).unwrap() == symbol
        })
        .expect("the checkpoint contains the fixture's symbol payload");
    let encoded = member[1];
    let offset = encoded.as_ptr() as usize - before.bytes().as_ptr() as usize;
    let mut tampered = before.bytes().to_vec();
    tampered[offset + encoded.len() - 1] ^= 1;
    assert!(
        RelationalNativeCheckpoint::inspect_bytes(&execution, &tampered).is_err(),
        "symbol payloads still undergo owner readmission and digest validation"
    );
}
