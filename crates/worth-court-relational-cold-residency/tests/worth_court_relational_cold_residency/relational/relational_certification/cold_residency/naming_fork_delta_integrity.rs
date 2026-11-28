//! Layer 2 semantic & adversarial courts for fork-delta `CHK24` naming sealing:
//! - Fork with zero changes stays completely isolated from post-fork parent
//!   creates, retirements, and group transitions across cold reopen AND
//!   carry-recapture into a second checkpoint.
//! - Multi-level fork chain (`main -> child -> grandchild`) with divergent
//!   edits, distinct retention floors, and checkpoint ACK rebinding matches
//!   the warm reference oracle 100% across two checkpoint generations.
//! - Tampered `delta_base` (missing target, self-cycle, 2-node cycle) fails
//!   closed read-free with `CorruptCheckpoint` and zero naming page reads.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;

type ExactResolutionView = (
    Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    Option<PersistentNameKey>,
    Option<Vec<PersistentNameIdentity>>,
);

fn resolve_status(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    name: PersistentNameIdentity,
) -> ExactResolutionView {
    let obs = observe_branch(runtime, branch);
    runtime
        .with_persistent_names_at_observation(request, &obs.observation(), |view| {
            let res = view
                .resolve(
                    PersistentNameKey::Entity(name),
                    &PersistentNameSelection::Exact,
                )
                .result;
            let grp = view.group_of_member(GROUP_SCHEME, &name);
            let members = grp.and_then(|g| view.group_members(g));
            (res, grp, members)
        })
        .unwrap()
}

fn assert_branches_match(
    request: &ExecutionRequest<'_, '_>,
    actual: &RelationalRuntime,
    expected: &RelationalRuntime,
    branches: &[&BranchId],
    names: &[PersistentNameIdentity],
    stage: &str,
) {
    for &branch in branches {
        for &name in names {
            assert_eq!(
                resolve_status(request, actual, branch, name),
                resolve_status(request, expected, branch, name),
                "branch {} disagreed on {name:?} ({stage})",
                branch.0
            );
        }
    }
}

fn assert_unknown(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    names: &[PersistentNameIdentity],
) {
    for &n in names {
        assert_eq!(
            resolve_status(request, runtime, branch, n),
            (
                Err(PersistentNameResolutionDenial::UnknownName {
                    name: PersistentNameKey::Entity(n),
                }),
                None,
                None,
            )
        );
    }
}

fn fork_branch_at(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: &BranchId,
    name: &str,
) -> BranchId {
    let target = BranchId(name.into());
    fork_retained_at(request, runtime, &observe_branch(runtime, source), &target).unwrap();
    target
}

#[test]
fn unchanged_fork_stays_isolated_from_post_fork_parent_advances_across_reopen_and_carry() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let pre_keys: Vec<String> = (0..16).map(|i| format!("pre-{i:02}")).collect();
    let (_, pre_e, pre_n) = create_named_entities(request, &producer, &main, &pre_keys);
    publish_group_partition(
        request,
        &producer,
        &main,
        pre_e[0],
        "pre-grp",
        vec![vec![
            (pre_n[0], AggregateMemberRole::Anchor),
            (pre_n[1], AggregateMemberRole::Interior),
            (pre_n[2], AggregateMemberRole::Interior),
        ]],
    );

    // Fork `silent` and make ZERO edits on `silent`.
    let silent = fork_branch_at(request, &producer, &main, "silent-fork");

    // Advance `main`: create new names, retire `pre_e[2]`, detach `pre_n[1]`.
    let (_, post_e, post_n) = create_named_entities(
        request,
        &producer,
        &main,
        &["post-1".into(), "post-2".into()],
    );
    delete_entity(request, &producer, &main, pre_e[2]);
    publish_group_partition(
        request,
        &producer,
        &main,
        post_e[0],
        "post-grp",
        vec![vec![
            (pre_n[1], AggregateMemberRole::Anchor),
            (post_n[0], AggregateMemberRole::Interior),
        ]],
    );

    // Capture C1 and reopen cold.
    let (c1, src1) = capture_external_checkpoint(request, &producer);
    let body1 = c1.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let silent_delta_base: Option<BranchId> =
        rows(field(field(field(body1, "manifest"), "naming"), "branches"))
            .into_iter()
            .find_map(|row| {
                let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
                (name == silent).then(|| rmp_serde::from_slice(field(row, "delta_base")).unwrap())
            })
            .expect("silent root present");
    assert_eq!(
        silent_delta_base.as_ref(),
        Some(&main),
        "silent fork should delta-seal against main"
    );

    let r1 = reopen_named(request, c1.clone(), src1.clone(), &medium, u64::MAX / 4);
    assert_branches_match(request, &r1, &producer, &[&silent], &pre_n, "silent C1");
    assert_unknown(request, &r1, &silent, &post_n);
    let all1: Vec<_> = pre_n.iter().chain(&post_n).copied().collect();
    assert_branches_match(request, &r1, &producer, &[&main], &all1, "main C1");

    let (_, post2_e, post2_n) =
        create_named_entities(request, &r1, &main, &["post-3".into(), "post-4".into()]);
    publish_group_partition(
        request,
        &r1,
        &main,
        post2_e[0],
        "post2-grp",
        vec![vec![
            (post2_n[0], AggregateMemberRole::Anchor),
            (post2_n[1], AggregateMemberRole::Interior),
        ]],
    );
    let (c2, src2) = capture_external_carrying(request, &r1, &src1);
    let r2 = reopen_named(request, c2, src2, &medium, u64::MAX / 4);
    assert_branches_match(request, &r2, &producer, &[&silent], &pre_n, "silent C2");
    assert_unknown(request, &r2, &silent, &post_n);
    assert_unknown(request, &r2, &silent, &post2_n);
    let all2: Vec<_> = all1.into_iter().chain(post2_n).collect();
    assert_branches_match(request, &r2, &r1, &[&main], &all2, "main C2");
}

#[test]
fn multilevel_fork_chain_with_divergent_edits_and_retention_floors_matches_warm_oracle() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut all_names = Vec::new();
    let (_, e0, n0) = create_named_entities(
        request,
        &producer,
        &main,
        &["m-0".into(), "m-1".into(), "m-2".into()],
    );
    all_names.extend_from_slice(&n0);
    publish_group_partition(
        request,
        &producer,
        &main,
        e0[0],
        "m-grp-0",
        vec![vec![
            (n0[0], AggregateMemberRole::Anchor),
            (n0[1], AggregateMemberRole::Interior),
        ]],
    );

    let child = fork_branch_at(request, &producer, &main, "chain-child");
    let (_, ec, nc) =
        create_named_entities(request, &producer, &child, &["c-0".into(), "c-1".into()]);
    all_names.extend_from_slice(&nc);
    delete_entity(request, &producer, &child, e0[2]);
    publish_group_partition(
        request,
        &producer,
        &child,
        ec[0],
        "c-grp",
        vec![vec![
            (nc[0], AggregateMemberRole::Anchor),
            (nc[1], AggregateMemberRole::Interior),
        ]],
    );

    let grandchild = fork_branch_at(request, &producer, &child, "chain-grandchild");
    let (_, eg, ng) = create_named_entities(
        request,
        &producer,
        &grandchild,
        &["g-0".into(), "g-1".into()],
    );
    all_names.extend_from_slice(&ng);
    publish_group_partition(
        request,
        &producer,
        &grandchild,
        eg[0],
        "g-grp",
        vec![vec![
            (n0[0], AggregateMemberRole::Anchor),
            (ng[0], AggregateMemberRole::Interior),
        ]],
    );

    // Advance `main` and raise retention floor ONLY on `main` (not on `child`/`grandchild`).
    let (m_adv, em_adv, nm_adv) = create_named_entities(
        request,
        &producer,
        &main,
        &["m-adv-0".into(), "m-adv-1".into()],
    );
    all_names.extend_from_slice(&nm_adv);
    delete_entity(request, &producer, &main, em_adv[1]);
    producer
        .advance_naming_retention_floor(&main, m_adv.commit.commit_id)
        .unwrap();

    let (c1, src1) = capture_external_checkpoint(request, &producer);
    let r1 = reopen_named(request, c1.clone(), src1.clone(), &medium, u64::MAX / 4);
    assert_branches_match(
        request,
        &r1,
        &producer,
        &[&main, &child, &grandchild],
        &all_names,
        "C1 reopen",
    );
    // Mutate all three branches after cold reopen, raise child's floor on r1,
    // carry-recapture C2, and verify both isolation and floor independence.
    let child_pre_floor = observe_branch(&r1, &child).observation().clone();
    let (_, _, nm2) = create_named_entities(request, &r1, &main, &["m-c2".into()]);
    let (c_post, _, nc2) = create_named_entities(request, &r1, &child, &["c-c2".into()]);
    let (_, _, ng2) = create_named_entities(request, &r1, &grandchild, &["g-c2".into()]);
    all_names.extend(nm2.into_iter().chain(nc2).chain(ng2));
    r1.advance_naming_retention_floor(&child, c_post.commit.commit_id)
        .unwrap();
    let child_pruned = r1
        .with_persistent_names_at_observation(request, &child_pre_floor, |view| {
            view.resolve(
                PersistentNameKey::Entity(n0[0]),
                &PersistentNameSelection::Exact,
            )
            .result
        })
        .unwrap();
    assert!(
        matches!(
            child_pruned,
            Err(PersistentNameResolutionDenial::BasisPruned { .. })
        ),
        "child must enforce its raised retention floor with BasisPruned, got {child_pruned:?}"
    );

    let (c2, src2) = capture_external_carrying(request, &r1, &src1);
    let r2 = reopen_named(request, c2, src2, &medium, u64::MAX / 4);
    assert_branches_match(
        request,
        &r2,
        &r1,
        &[&main, &child, &grandchild],
        &all_names,
        "C2 carry reopen",
    );
}

#[test]
fn tampered_delta_base_cycle_or_missing_target_refuses_corrupt_checkpoint_read_free() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, _) = create_named_entities(request, &producer, &main, &["t-0".into(), "t-1".into()]);
    let branch_b = fork_branch_at(request, &producer, &main, "tamper-b");
    let (_, _, _) = create_named_entities(request, &producer, &branch_b, &["tb-0".into()]);
    let (c1, _) = capture_external_checkpoint(request, &producer);

    let tamper_delta_bases = |edits: &[(&BranchId, Option<&BranchId>)]| {
        let body = c1.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
        let (manifest, naming) = (
            field(body, "manifest"),
            field(field(body, "manifest"), "naming"),
        );
        let edited: Vec<Vec<u8>> = rows(field(naming, "branches"))
            .into_iter()
            .map(|row| {
                let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
                match edits.iter().find(|(b, _)| **b == name) {
                    Some(&(_, repl)) => {
                        replace_field(row, "delta_base", &rmp_serde::to_vec(&repl).unwrap())
                    }
                    None => row.to_vec(),
                }
            })
            .collect();
        let refs: Vec<&[u8]> = edited.iter().map(Vec::as_slice).collect();
        let new_naming = replace_field(naming, "branches", &array(&refs));
        let new_body = replace_field(
            body,
            "manifest",
            &replace_field(manifest, "naming", &new_naming),
        );
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat())
    };

    let missing = BranchId("does-not-exist".into());
    for (label, edits) in [
        ("missing delta_base", vec![(&branch_b, Some(&missing))]),
        ("self-cycle delta_base", vec![(&branch_b, Some(&branch_b))]),
        (
            "2-branch cycle delta_base",
            vec![(&branch_b, Some(&main)), (&main, Some(&branch_b))],
        ),
    ] {
        super::naming_fork_delta_provenance::assert_corrupt_read_free(
            request,
            &medium,
            &c1,
            tamper_delta_bases(&edits),
            label,
        );
    }
}
