//! Roots sharing the same authenticated members must agree on leaf facts.
use super::{
    fields, memory_runtime, request, rewrite_row, rewritten, sequence, BranchId, ExecutionRequest,
    RecoveryFailureClass, RelationalNativeCheckpoint, RelationalRecoveryStop, CHECKPOINT_MAGIC,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot, durable_publication::field_patch,
};

#[test]
fn shared_members_with_inconsistent_leaf_facts_refuse_the_manifest() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let main = runtime.main_branch_identity().branch_id().clone();
    create_in_partition(&execution, &runtime, &main, PartitionId(1), "shared");
    let sibling = BranchId("other-root".into());
    let (_, source) = runtime.observe_fork_source(&main).unwrap();
    runtime
        .fork_branch(&execution, sibling.clone(), source)
        .unwrap();
    create_in_partition(&execution, &runtime, &sibling, PartitionId(2), "diverge");
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
&execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap();
    native.inspect(&execution).unwrap();

    let envelope = fields(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap());
    let manifest = fields(envelope["manifest"]);
    let roots = sequence(manifest["roots"]);
    assert_eq!(roots.len(), 2);
    let first = fields(sequence(fields(roots[0])["regions"])[0]);
    let shared = sequence(fields(roots[1])["regions"])
        .iter()
        .position(|region| {
            let leaf = fields(region);
            leaf["head"] == first["head"] && leaf["history"] == first["history"]
        })
        .expect("the retained roots share their first partition's real members");

    for name in [
        "content_digest",
        "entity_slots",
        "relation_slots",
        "unavailable_records",
    ] {
        let forged = rewritten(native.bytes(), false, |manifest| {
            let roots = rewrite_row(&manifest["roots"], 1, |root| {
                let regions = rewrite_row(&root["regions"], shared, |leaf| {
                    let value = leaf.get_mut(name).unwrap();
                    if name == "content_digest" {
                        *value.last_mut().unwrap() ^= 1;
                    } else {
                        let count = rmp_serde::from_slice::<u64>(value).unwrap();
                        *value = rmp_serde::to_vec(&(count + 1)).unwrap();
                    }
                });
                root.insert("regions".into(), regions);
            });
            manifest.insert("roots".into(), roots);
        });
        let RelationalRecoveryStop::Durability(failure) =
            RelationalNativeCheckpoint::inspect_bytes(&execution, &forged).unwrap_err()
        else {
            panic!("forged {name} must fail at manifest validation");
        };
        assert_eq!(failure.class, RecoveryFailureClass::CorruptCheckpoint);
        assert_eq!(
            failure.detail, "shared region members disagree on their leaf facts",
            "{name}"
        );
    }
}

fn create_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    partition_id: PartitionId,
    key: &str,
) {
    let identity = runtime.branch_identity(branch).unwrap();
    let basis = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id,
                    kind_id: KindId(1),
                    client_key: key.into(),
                    fields: field_patch("name", "name", key),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}
