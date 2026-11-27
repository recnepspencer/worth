//! Multi-branch fixture with real persistent naming schemes and persistent
//! groups for `#[global_allocator]` complexity probes.
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    aggregate_identity::AggregateMemberRole,
    durability::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
        RelationalNativeCheckpoint,
    },
    history::BranchId,
    identity::{EntityId, KindId, PartitionId},
    lineage::{
        CommitNamingPlan, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
        PersistentNameIdentity, PersistentNameKey, PersistentNameOfRecord,
    },
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
    transactions::{
        CreateIntent, CreatedEntityRef, DeleteEntityIntent, EntityMutationIntent, EntitySpec,
        MutationIntent, RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    cold_recovery::members_source,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::field_patch,
};

pub(super) use super::super::bounded_naming_fixture::{
    durable_named_runtime as durable_runtime, group_scheme, observe_branch, ENTITY_SCHEME,
    GROUP_SCHEME,
};

fn commit_batch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    batch: WorkerIntentBatch,
    plan: Option<CommitNamingPlan>,
) -> Vec<EntityId> {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    if let Some(plan) = plan {
        tx.attach_naming_plan(plan);
    }
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    committed
        .changed_records
        .iter()
        .filter_map(|r| match r {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .collect()
}

pub fn create_named_batch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    keys: &[String],
) -> (Vec<EntityId>, Vec<PersistentNameIdentity>) {
    create_named_batch_in_partition(request, runtime, branch, PartitionId::main(), keys)
}

pub fn create_named_batch_in_partition(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    partition_id: PartitionId,
    keys: &[String],
) -> (Vec<EntityId>, Vec<PersistentNameIdentity>) {
    let mut batch = WorkerIntentBatch::new("alloc-named");
    let mut plan = CommitNamingPlan::default();
    for key in keys {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id,
            kind_id: KindId(1),
            client_key: key.as_str().into(),
            fields: field_patch("name", "name", "v"),
        })));
        plan.pending_births.push(PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(CreatedEntityRef {
                partition_id,
                kind_id: KindId(1),
                client_key: key.as_str().into(),
            }),
            scheme_id: ENTITY_SCHEME.to_owned(),
            occurrence_key: format!("occ.{key}"),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
        });
    }
    let entities = commit_batch(request, runtime, branch, batch, Some(plan));
    let after = observe_branch(runtime, branch);
    let names = runtime
        .with_persistent_names_at_observation(request, &after.observation(), |view| {
            entities
                .iter()
                .map(
                    |e| match view.name_of_record(LineageRecordTarget::Entity(*e)) {
                        PersistentNameOfRecord::Live {
                            name: PersistentNameKey::Entity(id),
                            ..
                        } => id,
                        other => panic!("expected live entity name, got {other:?}"),
                    },
                )
                .collect::<Vec<_>>()
        })
        .unwrap();
    (entities, names)
}

pub fn publish_group(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    touch_entity: EntityId,
    groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
) {
    let batch = WorkerIntentBatch::new("grp").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: touch_entity,
            fields: field_patch("name", "name", "g"),
        }),
    ));
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(group_scheme(), Vec::new(), groups);
    commit_batch(request, runtime, branch, batch, Some(plan));
}

pub fn delete_entities(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entities: &[EntityId],
) {
    let batch = entities
        .iter()
        .fold(WorkerIntentBatch::new("del"), |b, &entity_id| {
            b.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id },
            )))
        });
    commit_batch(request, runtime, branch, batch, None);
}

pub struct BuiltCheckpoint {
    pub medium: Arc<TestDurableMedium>,
    pub checkpoint: RelationalNativeCheckpoint,
    pub manifest_only: RelationalNativeCheckpoint,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub main: BranchId,
    pub sample_name: PersistentNameIdentity,
    pub all_names: Vec<PersistentNameIdentity>,
    pub all_group_keys: Vec<PersistentNameKey>,
    pub naming_page_count: usize,
}

/// Builds a multi-branch (`main` + `branch-b`) checkpoint whose live named
/// entity count, persistent group count, and multi-branch history all scale
/// proportionally with `scale` (`1x` vs `10x`) while `PartitionId::main()`
/// stays fixed-size so point reads and single-partition edits stay O(1).
pub fn build_multi_branch_checkpoint(
    request: &ExecutionRequest<'_, '_>,
    scale: usize,
) -> BuiltCheckpoint {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let mut all_names = Vec::new();
    let mut all_group_keys = Vec::new();

    // Fixed-size `PartitionId::main()` anchor entities for point reads + single edit.
    let seed_keys: Vec<String> = (0..12).map(|i| format!("seed-e{i:02}")).collect();
    let (seed_e, seed_n) = create_named_batch(request, &producer, &main, &seed_keys);
    publish_group(
        request,
        &producer,
        &main,
        seed_e[0],
        vec![vec![
            (seed_n[0], AggregateMemberRole::Anchor),
            (seed_n[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let sample_name = seed_n[0];
    all_names.extend_from_slice(&seed_n);

    // Fork `branch-b` before scaling bulk named entities & persistent groups.
    let branch_b = BranchId("branch-b".into());
    let services = producer.owner_component_services();
    let fork_basis = services
        .basis_port()
        .admit_branch_basis(&producer.main_branch_identity())
        .unwrap();
    let fork_src = services
        .fork_port()
        .admit_retained_fork_source(&fork_basis)
        .unwrap();
    services
        .fork_port()
        .fork_retained_branch(request, branch_b.clone(), fork_src)
        .unwrap();
    drop((fork_basis, services));

    // Scale BOTH live named entities AND persistent groups proportionally to `scale`
    // while batching 64 entities/commit so commit count stays modest.
    let bulk_batches = (scale * 4).max(4);
    let (mut main_bulk_e, mut main_bulk_n, mut b_bulk_e, mut b_bulk_n) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for batch_idx in 0..bulk_batches {
        let on_b = batch_idx % 4 == 0;
        let target_branch = if on_b { &branch_b } else { &main };
        let pid = PartitionId(1 + (batch_idx % 6) as u32);
        let keys: Vec<String> = (0..64)
            .map(|j| format!("bulk-{batch_idx:04}-{j:02}"))
            .collect();
        let (e, n) = create_named_batch_in_partition(request, &producer, target_branch, pid, &keys);
        if on_b {
            b_bulk_e.extend(e);
            b_bulk_n.extend(n);
        } else {
            all_names.extend_from_slice(&n);
            main_bulk_e.extend(e);
            main_bulk_n.extend(n);
        }
    }

    for round in 0..scale * 8 {
        let on_b = round % 4 == 0 && b_bulk_n.len() >= 4;
        let (target_branch, entities, names) = if on_b {
            (&branch_b, &b_bulk_e, &b_bulk_n)
        } else {
            (&main, &main_bulk_e, &main_bulk_n)
        };
        let window = 32.min(names.len() / 2);
        let offset = (round * 6) % (names.len() - window * 2 + 1).max(1);
        let mut groups = Vec::with_capacity(window);
        let flip = (round / 4 + round) % 2 == 1;
        for p in 0..window {
            let (x, y) = (offset + p * 2, offset + p * 2 + 1);
            if y >= names.len() {
                break;
            }
            let (a, m) = if flip {
                (names[y], names[x])
            } else {
                (names[x], names[y])
            };
            groups.push(vec![
                (a, AggregateMemberRole::Anchor),
                (m, AggregateMemberRole::Ordinary),
            ]);
        }
        publish_group(
            request,
            &producer,
            target_branch,
            entities[offset % entities.len()],
            groups,
        );
        if !on_b {
            let obs = observe_branch(&producer, &main);
            producer
                .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                    if let Some(key) = view.group_of_member(GROUP_SCHEME, &names[offset]) {
                        all_group_keys.push(key);
                    }
                })
                .unwrap();
        }
    }

    let checkpoint = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = members_source(&checkpoint);
    let naming_page_count = member_count(&source, 5);
    let manifest_only =
        RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec());
    BuiltCheckpoint {
        medium,
        checkpoint,
        manifest_only,
        source,
        main,
        sample_name,
        all_names,
        all_group_keys,
        naming_page_count,
    }
}

pub fn reopen_cold(
    request: &ExecutionRequest<'_, '_>,
    built: &BuiltCheckpoint,
    historical_budget: u64,
) -> RelationalRuntime {
    let mut runtime = durable_runtime(&built.medium, historical_budget);
    let ck = RelationalDurableCheckpointSource::new(
        built.checkpoint.covered_position().unwrap(),
        built.checkpoint.covered_branch_transitions().unwrap(),
        built.manifest_only.clone(),
    );
    let input = RelationalDurableRecoverySource::new(
        Some(ck),
        Vec::new(),
        Vec::new(),
        built.source.clone(),
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    runtime
}

pub fn member_count(source: &ImmutableCheckpointMemberSource, tag: u8) -> usize {
    source
        .members
        .iter()
        .filter(|(_, b)| {
            b.bytes().starts_with(b"WORTH-REL-CHUNK1\n") && b.bytes().get(17) == Some(&tag)
        })
        .count()
}
