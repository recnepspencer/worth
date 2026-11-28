//! Semantic courts for fork-delta `CHK24` multi-scheme flattening and deep
//! linear fork chains spanning multiple seal depth caps.

use std::collections::BTreeMap;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameIdentity, PersistentNameKey, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection, PersistentNameTarget,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint, immutable_source,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, naming_pages, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;

fn publish_scheme_group(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    touch: EntityId,
    scheme_id: &str,
    label: &str,
    groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>,
) {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new(label).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: touch,
                fields: field_patch("name", "name", label),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let scheme = AggregateGroupScheme::new(
        AggregateGroupSchemeId::new(scheme_id).unwrap(),
        AggregateContinuityRule::Anchor,
    );
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(scheme, Vec::new(), groups);
    tx.attach_naming_plan(plan);
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

#[test]
fn two_scheme_cold_delta_branch_flattens_cleanly_at_second_checkpoint() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["ts-0".into(), "ts-1".into(), "ts-2".into(), "ts-3".into()],
    );
    let child = BranchId("two-scheme-child".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &child,
    )
    .unwrap();
    let (s1, s2) = ("worth.delta.scheme.one.v1", "worth.delta.scheme.two.v1");
    publish_scheme_group(
        request,
        &producer,
        &child,
        entities[0],
        s1,
        "child-s1",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Interior),
        ]],
    );
    publish_scheme_group(
        request,
        &producer,
        &child,
        entities[1],
        s2,
        "child-s2",
        vec![vec![
            (names[2], AggregateMemberRole::Anchor),
            (names[3], AggregateMemberRole::Interior),
        ]],
    );

    let (c1, src1) = capture_external_checkpoint(request, &producer);
    let r1 = reopen_named(request, c1, src1.clone(), &medium, u64::MAX / 4);
    // Main adds its FIRST group state after C1 reopen, mixing cold naming/index
    // with resident groups so child carry is ineligible and must flatten cleanly.
    publish_group_partition(
        request,
        &r1,
        &main,
        entities[0],
        "main-first-grp",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[2], AggregateMemberRole::Interior),
        ]],
    );
    let (c2, src2) = capture_external_carrying(request, &r1, &src1);
    let r2 = reopen_named(request, c2, src2, &medium, u64::MAX / 4);

    let child_obs = observe_branch(&r2, &child);
    r2.with_persistent_names_at_observation(request, &child_obs.observation(), |view| {
        let g1 = view.group_of_member(s1, &names[0]).expect("s1 survives C2");
        assert_eq!(view.group_of_member(s1, &names[1]), Some(g1));
        assert_eq!(view.group_of_member(s1, &names[2]), None);
        let g2 = view.group_of_member(s2, &names[2]).expect("s2 survives C2");
        assert_eq!(view.group_of_member(s2, &names[3]), Some(g2));
        assert_eq!(view.group_of_member(s2, &names[0]), None);
        assert_eq!(view.group_of_member(GROUP_SCHEME, &names[0]), None);
    })
    .unwrap();
    let main_obs = observe_branch(&r2, &main);
    r2.with_persistent_names_at_observation(request, &main_obs.observation(), |view| {
        let gm = view
            .group_of_member(GROUP_SCHEME, &names[0])
            .expect("main group survives C2");
        assert_eq!(view.group_of_member(GROUP_SCHEME, &names[2]), Some(gm));
        assert_eq!(view.group_of_member(s1, &names[0]), None);
        assert_eq!(view.group_of_member(s2, &names[2]), None);
    })
    .unwrap();
}

#[test]
fn deep_linear_fork_chain_spanning_multiple_depth_caps_avoids_full_reseal() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut base_pairs = Vec::new();
    for b in 0..12usize {
        let keys: Vec<String> = (0..25)
            .map(|i| format!("deep-base-{b:02}-{i:02}"))
            .collect();
        let (_, e, n) = create_named_entities(request, &producer, &main, &keys);
        base_pairs.push((e[0], n[0]));
    }
    let (base_ck, _) = capture_external_checkpoint(request, &producer);
    let base_src = immutable_source(&base_ck, &medium);
    let base_bytes: usize = naming_pages(&base_src)
        .iter()
        .map(|l| base_src.members[l].bytes().len())
        .sum();

    let mut chain = vec![(main.clone(), base_pairs[0].0, base_pairs[0].1)];
    for hop in 1..=24usize {
        let prev = chain.last().unwrap().0.clone();
        let next = BranchId(format!("chain-hop-{hop:02}"));
        fork_retained_at(request, &producer, &observe_branch(&producer, &prev), &next).unwrap();
        let (_, e, n) =
            create_named_entities(request, &producer, &next, &[format!("hop-name-{hop:02}")]);
        chain.push((next, e[0], n[0]));
    }

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let src = immutable_source(&ck, &medium);
    let chain_bytes: usize = naming_pages(&src)
        .iter()
        .map(|l| src.members[l].bytes().len())
        .sum();
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let delta_bases: BTreeMap<BranchId, Option<BranchId>> =
        rows(field(field(field(body, "manifest"), "naming"), "branches"))
            .into_iter()
            .map(|row| {
                let b: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
                let d: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
                (b, d)
            })
            .collect();
    for (hop, (branch, _, _)) in chain.iter().enumerate() {
        let mut depth = 0usize;
        let mut cur = branch;
        while let Some(Some(parent)) = delta_bases.get(cur) {
            depth += 1;
            cur = parent;
        }
        assert!(
            depth <= 8,
            "hop {hop} ({}) exceeded seal depth cap: {depth}",
            branch.0
        );
        if hop > 0 {
            assert!(
                delta_bases.get(branch).and_then(|d| d.as_ref()).is_some(),
                "hop {hop} must remain delta-sealed without re-sealing full base table"
            );
        }
    }
    assert!(
        (chain_bytes as f64) < (base_bytes as f64) * 2.5,
        "24-hop linear fork chain must stay O(names + hops): base={base_bytes} B, chain={chain_bytes} B"
    );

    let reopened = reopen_named(request, ck, src, &medium, u64::MAX / 4);
    for (hop, (branch, own_entity, own_name)) in chain.iter().enumerate() {
        let obs = observe_branch(&reopened, branch);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                let own_res = view
                    .resolve(
                        PersistentNameKey::Entity(*own_name),
                        &PersistentNameSelection::Exact,
                    )
                    .result;
                assert!(
                    matches!(
                        own_res,
                        Ok(PersistentNameResolution::Exact(
                            PersistentNameTarget::Entity { entity, .. }
                        )) if entity == *own_entity
                    ),
                    "hop {hop}: expected Exact({own_entity:?}), got {own_res:?}"
                );
                if hop + 1 < chain.len() {
                    let later = chain[hop + 1].2;
                    assert_eq!(
                        view.resolve(
                            PersistentNameKey::Entity(later),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                        Err(PersistentNameResolutionDenial::UnknownName {
                            name: PersistentNameKey::Entity(later),
                        })
                    );
                }
            })
            .unwrap();
    }
}

#[test]
fn deep_sibling_fork_forest_seals_in_bounded_depth_and_reopens_exact() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    for b in 0..8usize {
        let keys: Vec<String> = (0..25).map(|i| format!("sf-base-{b:02}-{i:02}")).collect();
        let _ = create_named_entities(request, &producer, &main, &keys);
    }
    let (base_ck, _) = capture_external_checkpoint(request, &producer);
    let base_src = immutable_source(&base_ck, &medium);
    let base_bytes: usize = naming_pages(&base_src)
        .iter()
        .map(|l| base_src.members[l].bytes().len())
        .sum();

    let mut parent = main.clone();
    let mut siblings = Vec::new();
    for depth in 1..=20usize {
        let basis = observe_branch(&producer, &parent);
        let spine = BranchId(format!("forest-spine-{depth:02}"));
        let sib = BranchId(format!("forest-sib-{depth:02}"));
        fork_retained_at(request, &producer, &basis, &spine).unwrap();
        fork_retained_at(request, &producer, &basis, &sib).unwrap();
        let (_, se, sn) =
            create_named_entities(request, &producer, &spine, &[format!("spine-n-{depth:02}")]);
        let (_, be, bn) =
            create_named_entities(request, &producer, &sib, &[format!("sib-n-{depth:02}")]);
        siblings.push((spine.clone(), se[0], sn[0], sib, be[0], bn[0]));
        parent = spine;
    }

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let src = immutable_source(&ck, &medium);
    let total_bytes: usize = naming_pages(&src)
        .iter()
        .map(|l| src.members[l].bytes().len())
        .sum();
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let delta_bases: BTreeMap<BranchId, Option<BranchId>> =
        rows(field(field(field(body, "manifest"), "naming"), "branches"))
            .into_iter()
            .map(|row| {
                let b: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
                let d: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
                (b, d)
            })
            .collect();
    for (spine, _, _, sib, _, _) in &siblings {
        for br in [spine, sib] {
            let mut d = 0usize;
            let mut cur = br;
            while let Some(Some(p)) = delta_bases.get(cur) {
                d += 1;
                cur = p;
            }
            assert!(d <= 8, "{} exceeded depth cap: {d}", br.0);
            assert!(delta_bases.get(br).and_then(|x| x.as_ref()).is_some());
        }
    }
    assert!(
        (total_bytes as f64) < (base_bytes as f64) * 3.5,
        "40-branch deep sibling forest must stay O(names + branches): base={base_bytes} B, total={total_bytes} B"
    );

    let reopened = reopen_named(request, ck, src, &medium, u64::MAX / 4);
    for (spine, se, sn, sib, be, bn) in [&siblings[0], siblings.last().unwrap()] {
        for (br, own_e, own_n, other_n) in [(spine, *se, *sn, *bn), (sib, *be, *bn, *sn)] {
            let obs = observe_branch(&reopened, br);
            reopened
                .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                    assert!(matches!(
                        view.resolve(
                            PersistentNameKey::Entity(own_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                        Ok(PersistentNameResolution::Exact(
                            PersistentNameTarget::Entity { entity, .. }
                        )) if entity == own_e
                    ));
                    assert_eq!(
                        view.resolve(
                            PersistentNameKey::Entity(other_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                        Err(PersistentNameResolutionDenial::UnknownName {
                            name: PersistentNameKey::Entity(other_n),
                        })
                    );
                })
                .unwrap();
        }
    }
}
