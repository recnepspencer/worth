//! Round 1 hardening courts for Store M4.3f (`[P1-5]`, `[P1-6]`, `[P1-8]`):
//! - Compaction preserves retired continuity for future deterministic recreation
//!   and enforces `BasisPruned` across all non-`resolve` observation accessors.
//! - Zero-edit `C1 -> ack -> C2` on a fresh runtime consolidates onto the cold
//!   base and carries 100% of naming pages with zero page reads.
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RelationalCheckpointArtifactPolicy, RelationalCheckpointMember, RelationalDurableColdSource,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
    PersistentNameIndexDenial, PersistentNameKey, PersistentNameResolution,
    PersistentNameSelection, RelationalPersistentNameReadDenial,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch,
    publish_group_partition, ENTITY_SCHEME, GROUP_SCHEME,
};

fn assert_pruned<T: std::fmt::Debug>(
    res: Result<T, RelationalPersistentNameReadDenial>,
    label: &str,
) {
    assert!(
        matches!(
            res,
            Err(RelationalPersistentNameReadDenial::IndexUnavailable(
                PersistentNameIndexDenial::BasisPruned { .. }
            ))
        ),
        "{label} below floor must refuse BasisPruned, got {res:?}"
    );
}

#[test]
fn compaction_preserves_retired_continuity_and_enforces_floor_on_all_accessors() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(
        request,
        &runtime,
        &main,
        &["keep-a".into(), "revive-me".into()],
    );
    let obs_c1 = observe_branch(&runtime, &main);
    publish_group_partition(
        request,
        &runtime,
        &main,
        e1[0],
        "grp-1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let group_key = runtime
        .with_persistent_names_at_observation(
            request,
            &observe_branch(&runtime, &main).observation(),
            |v| v.group_of_member(GROUP_SCHEME, &n1[0]).unwrap(),
        )
        .unwrap();
    delete_entity(request, &runtime, &main, e1[1]);
    let (c4, _, _) = create_named_entities(request, &runtime, &main, &["post-retire".into()]);
    let floor = c4.commit.commit_id;
    runtime
        .advance_naming_retention_floor(&main, floor)
        .unwrap();

    // [P1-6]: Every non-resolve accessor strictly below `floor` refuses with BasisPruned.
    assert_pruned(
        runtime.with_persistent_names_at_observation(request, &obs_c1.observation(), |v| {
            v.name_of_record(LineageRecordTarget::Entity(e1[0]))
        }),
        "name_of_record",
    );
    assert_pruned(
        runtime.with_persistent_names_at_observation(request, &obs_c1.observation(), |v| {
            v.group_of_member(GROUP_SCHEME, &n1[0])
        }),
        "group_of_member",
    );
    assert_pruned(
        runtime.with_persistent_names_at_observation(request, &obs_c1.observation(), |v| {
            v.groups_of_member(&n1[0]).len()
        }),
        "groups_of_member",
    );
    assert_pruned(
        runtime.with_persistent_names_at_observation(request, &obs_c1.observation(), |v| {
            v.group_members(group_key)
        }),
        "group_members",
    );
    assert_pruned(
        runtime.with_persistent_names_at_observation(request, &obs_c1.observation(), |v| {
            v.scheme_of(PersistentNameKey::Entity(n1[0]))
                .map(str::to_owned)
        }),
        "scheme_of",
    );

    // [P1-5]: Recreating the EXACT deterministic name retired before `floor` succeeds cleanly.
    let basis = observe_branch(&runtime, &main);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let batch = WorkerIntentBatch::new("revive").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: "revived-holder".into(),
            fields: field_patch("name", "name", "revived"),
        }),
    ));
    let mut plan = CommitNamingPlan::default();
    plan.pending_births.push(PendingCommitBirth {
        target: PendingBirthTarget::CreatedEntity(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: "revived-holder".into(),
        }),
        scheme_id: ENTITY_SCHEME.to_owned(),
        occurrence_key: "occ.revive-me".to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
    });
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    tx.attach_naming_plan(plan);
    let committed = tx
        .commit(request, &runtime)
        .expect("recreating deterministic name retired before floor must succeed");
    release_test_commit_snapshot(&runtime, &committed);

    let after = observe_branch(&runtime, &main);
    runtime
        .with_persistent_names_at_observation(request, &after.observation(), |view| {
            assert!(matches!(
                view.resolve(
                    PersistentNameKey::Entity(n1[1]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Ok(PersistentNameResolution::Exact(_))
            ));
        })
        .expect("branch name index stays healthy after post-compaction recreation");
}

#[test]
fn fresh_runtime_c1_ack_to_c2_carries_all_naming_pages_with_zero_reads() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    for b in 0..20usize {
        let keys: Vec<String> = (0..10).map(|i| format!("ack-{b:02}-{i}")).collect();
        let (_, e, n) = create_named_entities(request, &runtime, &main, &keys);
        publish_group_partition(
            request,
            &runtime,
            &main,
            e[0],
            &format!("ack-g-{b:02}"),
            vec![vec![
                (n[0], AggregateMemberRole::Anchor),
                (n[1], AggregateMemberRole::Ordinary),
            ]],
        );
    }

    let medium_source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let c1 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(medium_source.clone()),
            },
        )
        .unwrap();
    medium.file_checkpoint(c1.clone());
    request
        .run_native_with_checkpoint(|context| c1.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();

    let c1_dir = super::selected_directory::read_native(&c1);
    let c1_naming_pages: Vec<_> = c1_dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .chain(&c1_dir.naming.group_log)
        .map(|p| p.member)
        .collect();
    assert!(c1_naming_pages.len() >= 4);
    let c1_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let reads_before = runtime.cold_naming_observation().unwrap().faults;
    let c2 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &c1_members,
                source: Some(medium_source),
            },
        )
        .unwrap();
    let reads_after = runtime.cold_naming_observation().unwrap().faults;
    assert_eq!(
        reads_after, reads_before,
        "zero-edit C2 after ACK must fault 0 naming pages"
    );
    assert!(c1_naming_pages.iter().all(|locator| {
        c2.members().iter().any(|member| {
            matches!(member, RelationalCheckpointMember::Carried { locator: c } if *c == *locator)
        })
    }));
}
