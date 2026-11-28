//! Regression courts for retired-parent sibling/grandchild provenance,
//! warm historical fork naming headers after group-only commits, and
//! nearest-live-ancestor delta reuse across distinct retired intermediates.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameSelection, PersistentNameTarget,
};
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint, immutable_source,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, observe_branch, publish_group_partition,
    reopen_named,
};
use super::checkpoint_wire_fixture::{field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;

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
fn retired_parent_with_sibling_and_grandchild_checkpoints_and_reopens_cleanly() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, base_e, base_n) = create_named_entities(request, &producer, &main, &["base-0".into()]);
    let parent_p = BranchId("retired-p".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &parent_p,
    )
    .unwrap();
    let (_, p_e, p_n) = create_named_entities(request, &producer, &parent_p, &["p-0".into()]);

    let p_basis = observe_branch(&producer, &parent_p);
    let branch_a = BranchId("sib-a".into());
    let branch_b = BranchId("sib-b".into());
    fork_retained_at(request, &producer, &p_basis, &branch_a).unwrap();
    fork_retained_at(request, &producer, &p_basis, &branch_b).unwrap();

    let a_basis = observe_branch(&producer, &branch_a);
    let branch_c = BranchId("grandchild-c".into());
    fork_retained_at(request, &producer, &a_basis, &branch_c).unwrap();
    let (_, a_e, a_n) = create_named_entities(request, &producer, &branch_a, &["a-only".into()]);

    let p_id = producer.branch_identity(&parent_p).unwrap();
    assert!(producer
        .delete_branch(request, &p_id)
        .unwrap()
        .deleted()
        .is_some());

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    for row in rows(field(field(field(body, "manifest"), "naming"), "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        if br == branch_c {
            assert_ne!(
                base.as_ref(),
                Some(&branch_b),
                "grandchild C must never delta-seal against aunt/uncle sibling B"
            );
        }
    }

    let src = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, src, &medium, u64::MAX / 4);

    for (branch, has_a) in [(&branch_a, true), (&branch_b, false), (&branch_c, false)] {
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
                            PersistentNameKey::Entity(p_n[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    p_e[0],
                );
                let a_res = view
                    .resolve(
                        PersistentNameKey::Entity(a_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result;
                if has_a {
                    assert_exact(&a_res, a_e[0]);
                } else {
                    assert_unknown(&a_res, a_n[0]);
                }
            })
            .unwrap();
    }
}

#[test]
fn warm_head_and_historical_forks_after_group_only_commit_share_compatible_naming_headers() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(request, &producer, &main, &["hdr-e1".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e1[0],
        "group-only-transition",
        vec![vec![(n1[0], AggregateMemberRole::Anchor)]],
    );
    let cutoff = observe_branch(&producer, &main);

    let head_fork = BranchId("head-after-group".into());
    fork_retained_at(request, &producer, &cutoff, &head_fork).unwrap();

    let (_, e2, n2) = create_named_entities(request, &producer, &main, &["hdr-e2".into()]);

    let hist_fork = BranchId("hist-at-group-cutoff".into());
    fork_retained_at(request, &producer, &cutoff, &hist_fork).unwrap();

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let mut head_naming = None;
    let mut hist_naming = None;
    for row in rows(field(field(field(body, "manifest"), "naming"), "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if br == head_fork {
            head_naming = Some(field(row, "naming").to_vec());
        } else if br == hist_fork {
            hist_naming = Some(field(row, "naming").to_vec());
        }
    }
    assert_eq!(
        head_naming, hist_naming,
        "warm head and historical forks at the same post-group cutoff must share identical naming headers"
    );

    let src = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, src, &medium, u64::MAX / 4);
    for (branch, has_e2) in [(&main, true), (&head_fork, false), (&hist_fork, false)] {
        let obs = observe_branch(&reopened, branch);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(n1[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    e1[0],
                );
                let r2 = view
                    .resolve(
                        PersistentNameKey::Entity(n2[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result;
                if has_e2 {
                    assert_exact(&r2, e2[0]);
                } else {
                    assert_unknown(&r2, n2[0]);
                }
            })
            .unwrap();
    }
}

#[test]
fn distinct_retired_intermediates_reuse_nearest_live_ancestor_without_resealing_full_tables() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let base_keys: Vec<String> = (0..300).map(|i| format!("ret-anc-{i:03}")).collect();
    let (_, base_e, base_n) = create_named_entities(request, &producer, &main, &base_keys);

    let mut leaves = Vec::new();
    for i in 0..10 {
        let mid = BranchId(format!("retired-mid-{i:02}"));
        let leaf = BranchId(format!("surviving-leaf-{i:02}"));
        fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
        fork_retained_at(request, &producer, &observe_branch(&producer, &mid), &leaf).unwrap();
        let (_, le, ln) =
            create_named_entities(request, &producer, &leaf, &[format!("leaf-own-{i:02}")]);
        let mid_id = producer.branch_identity(&mid).unwrap();
        assert!(producer
            .delete_branch(request, &mid_id)
            .unwrap()
            .deleted()
            .is_some());
        leaves.push((leaf, le[0], ln[0]));
    }

    let (ck, src) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming = field(field(body, "manifest"), "naming");
    let mut delta_leaves = 0usize;
    let mut unique_members = std::collections::BTreeMap::new();
    for row in rows(field(naming, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        if br.0.starts_with("surviving-leaf-") && base.is_some() {
            delta_leaves += 1;
        }
        for page in rows(field(row, "pages")) {
            let member = field(page, "member");
            let len: u64 = rmp_serde::from_slice(field(member, "length")).unwrap();
            unique_members.insert(member.to_vec(), len);
        }
    }
    let total_member_bytes: u64 = unique_members.values().sum();
    assert_eq!(
        delta_leaves, 10,
        "every surviving leaf across retired intermediates must delta-seal over main/sibling"
    );
    assert!(
        total_member_bytes < 400_000,
        "retired intermediates caused full-table reseal: {total_member_bytes} bytes"
    );

    let reopened = reopen_named(request, ck, src.clone(), &medium, u64::MAX / 4);
    for (idx, (leaf, own_e, own_n)) in leaves.iter().enumerate() {
        let obs = observe_branch(&reopened, leaf);
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
                            PersistentNameKey::Entity(base_n[299]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    base_e[299],
                );
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(*own_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    *own_e,
                );
                let other = leaves[(idx + 1) % leaves.len()].2;
                assert_unknown(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(other),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    other,
                );
            })
            .unwrap();
    }

    // Re-capture C2 after cold reopen to verify retired_forks provenance persists across reopen.
    let (ck2, src2) = capture_external_carrying(request, &reopened, &src);
    let reopened2 = reopen_named(request, ck2, src2, &medium, u64::MAX / 4);
    let obs0 = observe_branch(&reopened2, &leaves[0].0);
    reopened2
        .with_persistent_names_at_observation(request, &obs0.observation(), |view| {
            assert_exact(
                &view
                    .resolve(
                        PersistentNameKey::Entity(leaves[0].2),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                leaves[0].1,
            );
        })
        .unwrap();
}
