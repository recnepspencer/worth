//! End-to-end courts for fork-delta `CHK24` branch deletion provenance and
//! per-branch index retention-floor alias safety.

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, PendingBirthTarget, PendingCommitBirth, PersistentNameKey,
    PersistentNameResolution, PersistentNameResolutionDenial, PersistentNameSelection,
    PersistentNameTarget,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch, reopen_named,
    ENTITY_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;
use super::naming_fork_delta_provenance::assert_corrupt_read_free;

fn assert_exact(
    res: &Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    expected: worth_relational::facade::identity::EntityId,
) {
    assert!(
        matches!(
            res,
            Ok(PersistentNameResolution::Exact(
                PersistentNameTarget::Entity { entity, .. }
            )) if *entity == expected
        ),
        "expected Exact({expected:?}), got {res:?}"
    );
}

fn assert_unknown(
    res: &Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    name: worth_relational::facade::lineage::PersistentNameIdentity,
) {
    assert_eq!(
        res,
        &Err(PersistentNameResolutionDenial::UnknownName {
            name: PersistentNameKey::Entity(name),
        })
    );
}

#[test]
fn deleted_fork_and_deleted_parent_with_surviving_children_checkpoint_and_reopen_cleanly() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, base_e, base_n) = create_named_entities(
        request,
        &producer,
        &main,
        &["del-b0".into(), "del-b1".into()],
    );

    // Case (a): fork a branch, add a name, then delete that fork before checkpointing.
    let dead_fork = BranchId("deleted-leaf-fork".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &dead_fork,
    )
    .unwrap();
    let (_, _, dead_n) =
        create_named_entities(request, &producer, &dead_fork, &["dead-only".into()]);
    let dead_id = producer.branch_identity(&dead_fork).unwrap();
    assert!(producer
        .delete_branch(request, &dead_id)
        .unwrap()
        .deleted()
        .is_some());

    // Case (b): fork `mid`, fork two live children from `mid`, then delete `mid`.
    let mid = BranchId("deleted-mid-parent".into());
    fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
    let (_, mid_e, mid_n) = create_named_entities(request, &producer, &mid, &["mid-0".into()]);
    let child_a = BranchId("surviving-child-a".into());
    let child_b = BranchId("surviving-child-b".into());
    let mid_basis = observe_branch(&producer, &mid);
    fork_retained_at(request, &producer, &mid_basis, &child_a).unwrap();
    fork_retained_at(request, &producer, &mid_basis, &child_b).unwrap();
    let (_, ca_e, ca_n) =
        create_named_entities(request, &producer, &child_a, &["child-a-0".into()]);
    let (_, cb_e, cb_n) =
        create_named_entities(request, &producer, &child_b, &["child-b-0".into()]);
    let mid_id = producer.branch_identity(&mid).unwrap();
    assert!(producer
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    for row in rows(field(field(field(body, "manifest"), "naming"), "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        assert_ne!(br, dead_fork, "deleted leaf fork must not be emitted");
        assert_ne!(br, mid, "deleted parent must not be emitted");
        assert_ne!(base.as_ref(), Some(&dead_fork));
        assert_ne!(base.as_ref(), Some(&mid));
    }

    let src = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, src, &medium, u64::MAX / 4);

    let main_obs = observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &main_obs.observation(), |view| {
            assert_exact(
                &view
                    .resolve(
                        PersistentNameKey::Entity(base_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                base_e[0],
            );
            assert_unknown(
                &view
                    .resolve(
                        PersistentNameKey::Entity(dead_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                dead_n[0],
            );
            assert_unknown(
                &view
                    .resolve(
                        PersistentNameKey::Entity(mid_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                mid_n[0],
            );
        })
        .unwrap();

    for (branch, own_e, own_n, other_n) in [
        (&child_a, ca_e[0], ca_n[0], cb_n[0]),
        (&child_b, cb_e[0], cb_n[0], ca_n[0]),
    ] {
        let obs = observe_branch(&reopened, branch);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(base_n[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    base_e[0],
                );
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(mid_n[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    mid_e[0],
                );
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(own_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    own_e,
                );
                assert_unknown(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(other_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    other_n,
                );
            })
            .unwrap();
    }
}

#[test]
fn parent_compaction_past_fork_preserves_child_index_history_and_refuses_tampered_alias_read_free()
{
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(
        request,
        &producer,
        &main,
        &["ret-keep".into(), "ret-target".into()],
    );
    let obs_live = observe_branch(&producer, &main);
    delete_entity(request, &producer, &main, e1[1]);
    let obs_retired = observe_branch(&producer, &main);

    // Recreate the exact same deterministic name `n1[1]` on `main`.
    let basis = observe_branch(&producer, &main);
    let mut tx = producer
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let batch = WorkerIntentBatch::new("revive-target").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: "revived-e".into(),
            fields: field_patch("name", "name", "revived"),
        }),
    ));
    let mut plan = CommitNamingPlan::default();
    plan.pending_births.push(PendingCommitBirth {
        target: PendingBirthTarget::CreatedEntity(CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: "revived-e".into(),
        }),
        scheme_id: ENTITY_SCHEME.to_owned(),
        occurrence_key: "occ.ret-target".to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
    });
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    tx.attach_naming_plan(plan);
    let c3 = tx.commit(request, &producer).unwrap();
    release_test_commit_snapshot(&producer, &c3);
    let obs_revived = observe_branch(&producer, &main);

    // Fork `child` at `obs_revived` so `child` and `main` share head `c3`.
    let child = BranchId("uncompacted-child".into());
    fork_retained_at(request, &producer, &obs_revived, &child).unwrap();
    // Compact ONLY `main` to `c3`, pruning `main`'s pre-`c3` history.
    producer
        .advance_naming_retention_floor(&main, c3.commit.commit_id)
        .unwrap();

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    for row in rows(field(naming, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        let pages = rows(field(row, "pages"));
        if br == child {
            assert!(
                !(pages.is_empty() && base.as_ref() == Some(&main)),
                "uncompacted child must NOT zero-page alias parent with newer retention_floor"
            );
            assert_ne!(
                base.as_ref(),
                Some(&main),
                "uncompacted child must NOT pick compacted parent as index delta_base"
            );
        }
    }

    // On `producer`, `main` refuses historical reads below its compacted floor.
    let main_pruned = producer
        .with_persistent_names_at_observation(request, &obs_live.observation(), |view| {
            view.resolve(
                PersistentNameKey::Entity(n1[1]),
                &PersistentNameSelection::Exact,
            )
            .result
        })
        .unwrap();
    assert!(
        matches!(
            main_pruned,
            Err(PersistentNameResolutionDenial::BasisPruned { .. })
        ),
        "main must refuse historical basis below compacted floor, got {main_pruned:?}"
    );

    // Cold reopen succeeds cleanly and preserves both branches' heads and floors:
    // advancing `child`'s floor to `obs_retired` succeeds without pruning `c3`,
    // and both branches resolve `n1[1]` at head.
    let src = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck.clone(), src, &medium, u64::MAX / 4);
    let child_head = observe_branch(&reopened, &child);
    let main_head = observe_branch(&reopened, &main);
    for obs in [&child_head, &main_head] {
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                assert!(matches!(
                    view.resolve(
                        PersistentNameKey::Entity(n1[1]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                    Ok(PersistentNameResolution::Exact(_))
                ));
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(n1[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    e1[0],
                );
            })
            .unwrap();
    }
    let _ = (obs_retired, obs_live);

    // Read-free adversarial manifest test: tamper `child` to zero-page alias `main`
    // even though `main.index.retention_floor > child.index.retention_floor`.
    let tampered_rows: Vec<Vec<u8>> = rows(field(naming, "branches"))
        .into_iter()
        .map(|row| {
            let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if br == child {
                let r = replace_field(row, "pages", &array(&[]));
                replace_field(&r, "delta_base", &rmp_serde::to_vec(&Some(&main)).unwrap())
            } else {
                row.to_vec()
            }
        })
        .collect();
    let refs: Vec<&[u8]> = tampered_rows.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(naming, "branches", &array(&refs));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let tampered =
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat());
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        tampered,
        "zero-page index alias over parent with newer retention_floor",
    );
}
