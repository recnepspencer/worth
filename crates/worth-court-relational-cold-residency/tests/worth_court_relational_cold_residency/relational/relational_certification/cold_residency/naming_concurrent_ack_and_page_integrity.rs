//! Cold-residency courts for concurrent ACK rebinding, detached fork atomicity,
//! unnamed retirement truncation, and stripped domain page-run refusal:
//! - Concurrent readers on a post-capture fork during C2 ACK never observe torn state.
//! - Concurrent `RelationalRuntime::fork()` racing C2 ACK preserves inherited names.
//! - Retired unnamed entities are never resurrected in a cold historical fork.
//! - Tampered `CHK24` stripping naming, index, or group pages refuses `CorruptCheckpoint`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointMember, RelationalDurableMemberLocator,
    RelationalNativeCheckpoint,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolution, PersistentNameSelection,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch,
    publish_group_partition,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_ack_isolation_and_rebind::{ack_retry_busy, capture_only, try_ack};
use super::naming_branch_root_and_dissolve::fork_retained_at;
use super::naming_historical_fork_integrity::durable_mixed_runtime;

fn strip_domain_pages(
    ck: &RelationalNativeCheckpoint,
    target_branch: &BranchId,
    tags: &[u8],
) -> RelationalNativeCheckpoint {
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let branch_rows = rows(field(naming, "branches"));
    let mut edited: Vec<Vec<u8>> = Vec::with_capacity(branch_rows.len());
    let mut stripped_any = false;
    for row in branch_rows {
        let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if &name == target_branch {
            let pages = rows(field(row, "pages"));
            let mut kept = Vec::new();
            for page in pages {
                let first: Vec<u8> = rmp_serde::from_slice(field(page, "first")).unwrap();
                if tags.contains(&first[0]) {
                    stripped_any = true;
                } else {
                    kept.push(page);
                }
            }
            edited.push(replace_field(row, "pages", &array(&kept)));
        } else {
            edited.push(row.to_vec());
        }
    }
    assert!(
        stripped_any,
        "target branch must have at least one page in {tags:?} to strip"
    );
    let refs: Vec<&[u8]> = edited.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(naming, "branches", &array(&refs));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat())
}

fn setup_post_capture_fork(
    request: &ExecutionRequest<'_, '_>,
    tag: &str,
) -> (
    Arc<TestDurableMedium>,
    Arc<worth_relational::facade::runtime::RelationalRuntime>,
    BranchId,
    BranchId,
    worth_relational::facade::lineage::PersistentNameIdentity,
    RelationalNativeCheckpoint,
) {
    let medium = TestDurableMedium::new();
    let runtime = Arc::new(durable_named_runtime(&medium, u64::MAX / 4));
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(request, &runtime, &main, &[format!("{tag}-k")]);
    let ck1 = capture_only(request, &runtime, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    let b1 = BranchId(format!("{tag}-b1"));
    let main_c1 = observe_branch(&runtime, &main);
    fork_retained_at(request, &runtime, &main_c1, &b1).unwrap();

    delete_entity(request, &runtime, &main, e1[0]);
    let ck1_members: Vec<_> = ck1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let ck2 = capture_only(request, &runtime, &medium, &ck1_members);
    medium.file_checkpoint(ck2.clone());

    let b2 = BranchId(format!("{tag}-b2"));
    let b1_obs = observe_branch(&runtime, &b1);
    fork_retained_at(request, &runtime, &b1_obs, &b2).unwrap();
    (medium, runtime, main, b2, n1[0], ck2)
}

fn assert_b2_exact(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    b2: &BranchId,
    name: worth_relational::facade::lineage::PersistentNameIdentity,
) {
    let obs = observe_branch(runtime, b2);
    runtime
        .with_persistent_names_at_observation(request, &obs.observation(), |v| {
            let res = v
                .resolve(
                    PersistentNameKey::Entity(name),
                    &PersistentNameSelection::Exact,
                )
                .result;
            assert!(
                matches!(res, Ok(PersistentNameResolution::Exact(_))),
                "{res:?}"
            );
        })
        .expect("b2 exact resolution must succeed");
}

#[test]
fn concurrent_ack_rebind_never_breaks_live_readers_on_post_capture_fork() {
    for round in 0..8usize {
        let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let request = &ExecutionRequest::serial(&host);
        let (_medium, runtime, _main, b2, name, ck2) =
            setup_post_capture_fork(request, &format!("r4-p1-1-{round}"));

        let barrier = Arc::new(Barrier::new(2));
        let done = Arc::new(AtomicBool::new(false));
        let (rt_reader, b2_reader, b_reader, done_reader) = (
            Arc::clone(&runtime),
            b2.clone(),
            Arc::clone(&barrier),
            Arc::clone(&done),
        );
        let reader = thread::spawn(move || {
            let h = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
            let req = ExecutionRequest::serial(&h);
            b_reader.wait();
            let mut checks = 0usize;
            while !done_reader.load(Ordering::Acquire) || checks < 16 {
                let live = rt_reader.branch_live_persistent_names(&b2_reader).unwrap();
                assert!(
                    live.contains_key(&name),
                    "round {round} check {checks}: b2 lost name"
                );
                assert_b2_exact(&req, &rt_reader, &b2_reader, name);
                checks += 1;
            }
        });

        barrier.wait();
        ack_retry_busy(request, &ck2);
        done.store(true, Ordering::Release);
        reader.join().unwrap();
    }
}

#[test]
fn concurrent_runtime_fork_during_c2_ack_preserves_b2_inherited_names() {
    for round in 0..12usize {
        let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let request = &ExecutionRequest::serial(&host);
        let (medium, runtime, main, b2, name, ck2) =
            setup_post_capture_fork(request, &format!("r4-p1-2-{round}"));

        let barrier = Arc::new(Barrier::new(2));
        let (rt_fork, b_fork) = (Arc::clone(&runtime), Arc::clone(&barrier));
        let forker = thread::spawn(move || {
            b_fork.wait();
            rt_fork.fork().expect("detached runtime fork must succeed")
        });

        barrier.wait();
        ack_retry_busy(request, &ck2);
        let detached = forker.join().unwrap();

        // Advance original runtime further and ACK C3 so C1/C2 are retired on parent.
        let _ = create_named_entities(request, &runtime, &main, &[format!("r4-p1-2-c3-{round}")]);
        let ck2_members: Vec<_> = ck2
            .members()
            .iter()
            .map(RelationalCheckpointMember::locator)
            .collect();
        let ck3 = capture_only(request, &runtime, &medium, &ck2_members);
        medium.file_checkpoint(ck3.clone());
        try_ack(request, &ck3).unwrap();

        // Detached fork must keep b2's inherited name intact regardless of race order.
        assert!(
            detached
                .branch_live_persistent_names(&b2)
                .expect("detached b2 must stay readable")
                .contains_key(&name),
            "round {round}: detached fork lost b2 inherited name when racing C2 ACK"
        );
        assert_b2_exact(request, &detached, &b2, name);
    }
}

#[test]
fn retired_unnamed_entity_not_resurrected_in_cold_historical_fork() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    // C1: create unnamed entity (KindId(3)).
    let basis0 = observe_branch(&producer, &main);
    let mut tx = producer
        .begin_branch_transaction(&basis0, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("r4-unnamed-c1").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: "r4-unnamed-1".into(),
                fields: field_patch("name", "name", "r4-unnamed-1"),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let c1_res = tx.commit(request, &producer).unwrap();
    release_test_commit_snapshot(&producer, &c1_res);
    let unnamed_entity: EntityId = c1_res
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .unwrap();
    let basis_c1 = observe_branch(&producer, &main);

    // C2: delete the unnamed entity, retiring its unnamed lineage.
    delete_entity(request, &producer, &main, unnamed_entity);
    let basis_c2 = observe_branch(&producer, &main);

    // C3: create a named entity, checkpoint & ACK so `producer` transitions to
    // cold-backed naming (`full_history = None`), then fork at C1 and C2.
    let _ = create_named_entities(request, &producer, &main, &["r4-c3-named".into()]);
    let ck1 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    // Fork historical children at C1 (alive) and C2 (retired) from cold runtime.
    let (child_c1, child_c2) = (
        BranchId("r4-unnamed-child-c1".into()),
        BranchId("r4-unnamed-child-c2".into()),
    );
    fork_retained_at(request, &producer, &basis_c1, &child_c1).unwrap();
    fork_retained_at(request, &producer, &basis_c2, &child_c2).unwrap();

    // Domain-1 rows are LINEAGE_HOLDER (tag 1) and LIVE_LINEAGE (tag 4).
    let ck1_members: Vec<_> = ck1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let ck2 = capture_only(request, &producer, &medium, &ck1_members);
    medium.file_checkpoint(ck2.clone());
    try_ack(request, &ck2).unwrap();
    drop(producer);

    let mut reopened = durable_mixed_runtime(&medium);
    let mode = RecoveryVerificationMode::NormalRecoveryVerification;
    let plan = reopened
        .durability()
        .recovery_plan_from_source(request, medium.recovery_source(), mode)
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    let members: BTreeMap<_, _> = ck1
        .members()
        .iter()
        .chain(ck2.members())
        .filter_map(|m| match m {
            RelationalCheckpointMember::New { locator, bytes } => Some((*locator, bytes.bytes())),
            RelationalCheckpointMember::Carried { .. } => None,
        })
        .collect();
    let body = ck2.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let branch_rows = rows(field(field(field(body, "manifest"), "naming"), "branches"));
    let naming_page_summary = |target: &BranchId| -> (u8, u8, u32) {
        let mut chain = Vec::new();
        let mut cur = Some(target.clone());
        while let Some(branch) = cur {
            let row = branch_rows
                .iter()
                .find(|r| rmp_serde::from_slice::<BranchId>(field(r, "branch")).unwrap() == branch)
                .expect("child root present");
            chain.push(*row);
            cur = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        }
        let mut live: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        for row in chain.into_iter().rev() {
            for p in rows(field(row, "pages")) {
                let first: Vec<u8> = rmp_serde::from_slice(field(p, "first")).unwrap();
                if !(1..=5).contains(&first[0]) {
                    continue;
                }
                let loc: RelationalDurableMemberLocator =
                    rmp_serde::from_slice(field(p, "member")).unwrap();
                let (entries,): (Vec<(Vec<u8>, Vec<u8>)>,) =
                    rmp_serde::from_slice(&members[&loc][18..]).unwrap();
                for (k, v) in entries {
                    if v.first() == Some(&2) {
                        live.remove(&k);
                    } else {
                        live.insert(k, v);
                    }
                }
            }
        }
        let lo = live.keys().map(|k| k[0]).min().unwrap_or(u8::MAX);
        let hi = live.keys().map(|k| k[0]).max().unwrap_or(0);
        (lo, hi, live.len() as u32)
    };
    assert_eq!(naming_page_summary(&child_c1), (1, 4, 2));
    assert_eq!(naming_page_summary(&child_c2), (1, 1, 1));
}

#[test]
fn tampered_chk24_stripping_naming_index_or_group_pages_refuses_corrupt_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(
        request,
        &producer,
        &main,
        &["p1-4-a".into(), "p1-4-b".into()],
    );
    let g1 = vec![vec![
        (n1[0], AggregateMemberRole::Anchor),
        (n1[1], AggregateMemberRole::Interior),
    ]];
    publish_group_partition(request, &producer, &main, e1[0], "p1-4-g1", g1);
    let basis_c1 = observe_branch(&producer, &main);
    let (_, e2, n2) = create_named_entities(request, &producer, &main, &["p1-4-c".into()]);
    let g2 = vec![vec![
        (n1[0], AggregateMemberRole::Anchor),
        (n2[0], AggregateMemberRole::Interior),
    ]];
    publish_group_partition(request, &producer, &main, e2[0], "p1-4-g2", g2);
    let child = BranchId("p1-4-hist-child".into());
    fork_retained_at(request, &producer, &basis_c1, &child).unwrap();
    let (ck, _source) = capture_external_checkpoint(request, &producer);

    // Strip each component's pages independently while leaving ALL headers intact!
    for (label, tags) in [
        ("naming-domain-1", &[1u8, 2, 3, 4, 5][..]),
        ("index-domain-2", &[6u8, 7, 8, 9, 10][..]),
        ("group-domain-3", &[0u8, 11, 12, 13][..]),
    ] {
        super::naming_fork_delta_provenance::assert_corrupt_read_free(
            request,
            &medium,
            &ck,
            strip_domain_pages(&ck, &main, tags),
            label,
        );
    }
}
