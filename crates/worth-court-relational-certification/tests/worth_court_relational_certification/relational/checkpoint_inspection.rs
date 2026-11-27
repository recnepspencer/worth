//! Independent authenticated-frame rewrites exercise the codec's closed schema.
use crate::messagepack_value_spans::{fields, sequence};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionAllocationPolicy, ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::{
    durability::{
        CheckpointRecordIdentity, RecordAllocationClass, RecoveryFailureClass,
        RelationalCheckpointArtifactPolicy::ExternalMembers, RelationalCheckpointMember,
        RelationalDurableMemberLocator, RelationalNativeCheckpoint, RelationalRecoveryStop,
    },
    history::BranchId,
    mvcc::RelationalTransactionIntent,
    transactions::WorkerIntentBatch,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    cold_recovery::members_source,
    durable_publication::{batch_create, memory_runtime},
    maintenance::relation_laws::{request, World},
    ordinary_records::create_entity,
};

mod allocation_bytes;
mod burned_symbol_frontier;
mod shared_members;
mod unknown_fields;

const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK24\n";

fn capture_external(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> RelationalNativeCheckpoint {
    let policy = ExternalMembers {
        current_members: &[],
        source: None,
    };
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            ExecutionAllocationPolicy::SystemAllocation,
            policy,
        )
        .unwrap()
}

fn original(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalNativeCheckpoint, [CheckpointRecordIdentity; 2]) {
    let world = World::new(execution, 1);
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let mut tx = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let batch = WorkerIntentBatch::new("pending").push(world.insert(0, 3, "pending-edge"));
    tx.push_batch(batch, execution.allocation_policy()).unwrap();
    let _pending = world
        .runtime
        .prepare_branch_transaction(execution, tx)
        .unwrap();
    let (entity, relation) = (world.nodes[0], world.edges[0].0);
    (
        capture_external(execution, &world.runtime),
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
            // CHK24 keeps the allocator's paged-directory roots in the manifest.
            let value = map(&stored["record_identity"], |identity| {
                let roots = map(&identity["roots"], edit);
                identity.insert("roots".into(), roots);
            });
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
    let decoded = RelationalNativeCheckpoint::inspect_bytes_with_source(
        &execution,
        before.bytes(),
        members_source(&before),
    )
    .unwrap();
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
    let identity = fields(checkpoint["record_identity"]);
    let mut compared = 0;
    for (key, old) in fields(identity["roots"]) {
        if matches!(
            key.as_str(),
            "append_frontiers" | "generation_high_water" | "partition_high_water"
        ) {
            continue;
        }
        // An absent table has no wider encoding; a present root does.
        let Some(equivalent) = wider(old) else {
            assert_eq!(old, [0xc0], "allocator root {key}");
            continue;
        };
        compared += 1;
        let after = rewritten(before.bytes(), true, |fields| {
            fields.insert(key.clone(), equivalent);
        });
        let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &after).unwrap();
        assert_ne!(
            decoded.model, after.model,
            "ignored allocator model field {key}"
        );
    }
    assert!(
        compared > 0,
        "the fixture carries a reusable or pending table"
    );
}
#[test]
fn restore_refuses_unknown_fields_before_publishing_any_model_state() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (original, _) = original(&execution);
    let mut control = World::empty_runtime(0);
    control
        .durability_recovery()
        .restore_native_checkpoint(&execution, &original, members_source(&original))
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
            } else if allocator {
                let root = map(&fields[owner], |root| {
                    root.insert("future_field".into(), vec![0xc0]);
                });
                fields.insert(owner.into(), root);
            } else {
                let rows = rewrite_row(&fields[owner], 0, |row| {
                    row.insert("future_field".into(), vec![0xc0]);
                });
                fields.insert(owner.into(), rows);
            }
        });
        let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes);
        let mut target = World::empty_runtime(0);
        let before = capture_external(&execution, &target);
        let denied = target
            .durability_recovery()
            .restore_native_checkpoint(&execution, &checkpoint, members_source(&checkpoint))
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
        let after = capture_external(&execution, &target);
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
        let (_, basis) = runtime
            .observe_branch(&runtime.branch_identity(&branch_id).unwrap())
            .unwrap();
        let mut tx = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            batch_create(branch_id.0.as_str()),
            execution.allocation_policy(),
        )
        .unwrap();
        let commit = tx.commit(&execution, &runtime).unwrap();
        expected.extend(changed_entities(&commit));
        release_test_commit_snapshot(&runtime, &commit);
    }
    let checkpoint = capture_external(&execution, &runtime);
    let inspected = RelationalNativeCheckpoint::inspect_bytes_with_source(
        &execution,
        checkpoint.bytes(),
        members_source(&checkpoint),
    )
    .unwrap();
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
