//! End-to-end court for composed prior-delta carry when intermediate branches
//! retire AFTER cold reopen (`C1` seals `main -> mid_i -> leaf_i` while `mid_i`
//! is live, reopen `C1`, delete all `mid_i`, edit `main` and `leaf_i`, seal `C2`
//! in `O(names + deltas)` without full-table reseal, and reopen `C2` cleanly).

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameSelection, PersistentNameTarget,
};
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch, reopen_named,
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

#[test]
fn post_reopen_intermediate_retirement_composes_delta_without_full_table_reseal() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let base_keys: Vec<String> = (0..300).map(|i| format!("post-ret-base-{i:03}")).collect();
    let (_, base_e, base_n) = create_named_entities(request, &producer, &main, &base_keys);

    let mut mids = Vec::new();
    let mut leaves = Vec::new();
    for i in 0..10usize {
        let mid = BranchId(format!("live-mid-{i:02}"));
        let leaf = BranchId(format!("post-ret-leaf-{i:02}"));
        fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
        let (_, me, mn) =
            create_named_entities(request, &producer, &mid, &[format!("mid-edit-{i:02}")]);
        let _ = delete_entity(request, &producer, &mid, base_e[i]);
        fork_retained_at(request, &producer, &observe_branch(&producer, &mid), &leaf).unwrap();
        let (_, le, ln) =
            create_named_entities(request, &producer, &leaf, &[format!("leaf-c1-{i:02}")]);
        mids.push((mid, me[0], mn[0]));
        leaves.push((leaf, le[0], ln[0]));
    }

    // C1 while every intermediate `mid_i` is still live.
    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let body1 = ck1.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    for row in rows(field(field(field(body1, "manifest"), "naming"), "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        if let Some(pos) = leaves.iter().position(|(l, _, _)| l == &br) {
            assert_eq!(
                base.as_ref(),
                Some(&mids[pos].0),
                "C1 leaf {br:?} should delta-seal over its live intermediate mid"
            );
        }
    }

    // Reopen C1 cold, then delete all 10 intermediates and mutate `main` + every `leaf_i`.
    let mut reopened1 = reopen_named(request, ck1, src1.clone(), &medium, u64::MAX / 4);
    for (mid, _, _) in &mids {
        let mid_id = reopened1.branch_identity(mid).unwrap();
        assert!(reopened1
            .delete_branch(request, &mid_id)
            .unwrap()
            .deleted()
            .is_some());
    }
    let (_, main_post_e, main_post_n) =
        create_named_entities(request, &reopened1, &main, &["main-post-c1".into()]);
    let mut leaf_post = Vec::new();
    for (i, (leaf, _, _)) in leaves.iter().enumerate() {
        let (_, pe, pn) =
            create_named_entities(request, &reopened1, leaf, &[format!("leaf-post-c1-{i:02}")]);
        leaf_post.push((pe[0], pn[0]));
    }

    // Capture C2: every leaf must compose its C1 delta chain over `main` without resealing full tables.
    let (ck2, src2) = capture_external_carrying(request, &reopened1, &src1);
    let body2 = ck2.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming2 = field(field(body2, "manifest"), "naming");
    let mut delta_over_main = 0usize;
    let mut unique_members = std::collections::BTreeMap::new();
    for row in rows(field(naming2, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        if br.0.starts_with("post-ret-leaf-") {
            assert_eq!(
                base.as_ref(),
                Some(&main),
                "C2 leaf {br:?} must compose its delta over surviving main"
            );
            delta_over_main += 1;
        }
        for page in rows(field(row, "pages")) {
            let member = field(page, "member");
            let len: u64 = rmp_serde::from_slice(field(member, "length")).unwrap();
            unique_members.insert(member.to_vec(), len);
        }
    }
    assert_eq!(delta_over_main, 10);
    let total_member_bytes: u64 = unique_members.values().sum();
    assert!(
        total_member_bytes < 400_000,
        "post-reopen intermediate retirement caused full-table reseal: {total_member_bytes} bytes"
    );

    // Reopen C2 and verify exact resolution across `main` and every `leaf_i`.
    let reopened2 = reopen_named(request, ck2, src2, &medium, u64::MAX / 4);
    let main_obs = observe_branch(&reopened2, &main);
    reopened2
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
            assert_exact(
                &view
                    .resolve(
                        PersistentNameKey::Entity(main_post_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                main_post_e[0],
            );
        })
        .unwrap();

    for (i, (leaf, leaf_c1_e, leaf_c1_n)) in leaves.iter().enumerate() {
        let obs = observe_branch(&reopened2, leaf);
        let (_, mid_e, mid_n) = &mids[i];
        let (post_e, post_n) = leaf_post[i];
        let sibling_n = leaves[(i + 1) % leaves.len()].2;
        reopened2
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                // Inherited untouched name from main.
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(base_n[299]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    base_e[299],
                );
                // Name created on the now-retired intermediate `mid_i`.
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(*mid_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    *mid_e,
                );
                // Inherited name deleted/tombstoned on the now-retired intermediate `mid_i`.
                assert!(
                    matches!(
                        view.resolve(
                            PersistentNameKey::Entity(base_n[i]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                        Ok(PersistentNameResolution::Retired { .. })
                    ),
                    "leaf {i} must preserve retired status of base_n[{i}] deleted on mid_{i}"
                );
                // C1 leaf name and post-C1 leaf name.
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(*leaf_c1_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    *leaf_c1_e,
                );
                assert_exact(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(post_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    post_e,
                );
                // Post-C1 edit on `main` and sibling leaf name must stay absent on `leaf_i`.
                assert_eq!(
                    view.resolve(
                        PersistentNameKey::Entity(main_post_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                    Err(PersistentNameResolutionDenial::UnknownName {
                        name: PersistentNameKey::Entity(main_post_n[0]),
                    })
                );
                assert_eq!(
                    view.resolve(
                        PersistentNameKey::Entity(sibling_n),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                    Err(PersistentNameResolutionDenial::UnknownName {
                        name: PersistentNameKey::Entity(sibling_n),
                    })
                );
            })
            .unwrap();
    }
}

#[test]
fn shared_retired_intermediate_rebases_once_and_aliases_peer_leaves() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let base_keys: Vec<String> = (0..300).map(|i| format!("fan-base-{i:03}")).collect();
    let (_, base_e, base_n) = create_named_entities(request, &producer, &main, &base_keys);

    // Fork `mid` from `main`, create B = 12 identical leaves of `mid`, then add
    // M = 60 names to `main` so C1 seals `mid` as a 60-tombstone delta over `main`
    // and all 12 leaves as 0-page aliases over `mid`.
    let mid = BranchId("fan-shared-mid".into());
    fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
    let mid_obs = observe_branch(&producer, &mid);
    let leaves: Vec<BranchId> = (0..12usize)
        .map(|i| BranchId(format!("fan-peer-leaf-{i:02}")))
        .collect();
    for leaf in &leaves {
        fork_retained_at(request, &producer, &mid_obs, leaf).unwrap();
    }
    let post_main_keys: Vec<String> = (0..60).map(|i| format!("fan-main-post-{i:03}")).collect();
    let (_, _, post_main_n) = create_named_entities(request, &producer, &main, &post_main_keys);

    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let mut reopened1 = reopen_named(request, ck1, src1.clone(), &medium, u64::MAX / 4);
    let mid_id = reopened1.branch_identity(&mid).unwrap();
    assert!(reopened1
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    // Capture C2: ONE surviving leaf rebases over `main`, and the 11 peers alias
    // that representative with 0 delta pages instead of repeating the M tombstones.
    let (ck2, src2) = capture_external_carrying(request, &reopened1, &src1);
    let body2 = ck2.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming2 = field(field(body2, "manifest"), "naming");
    let mut over_main = Vec::new();
    let mut peer_bases = Vec::new();
    let mut peer_pages = 0usize;
    for row in rows(field(naming2, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if !br.0.starts_with("fan-peer-leaf-") {
            continue;
        }
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        let page_count = rows(field(row, "pages")).len();
        if base.as_ref() == Some(&main) {
            over_main.push(br);
        } else {
            peer_bases.push((br, base, page_count));
            peer_pages += page_count;
        }
    }
    assert_eq!(
        over_main.len(),
        1,
        "exactly one representative leaf must rebase over main, got {over_main:?}"
    );
    let rep = &over_main[0];
    assert_eq!(peer_bases.len(), 11);
    for (br, base, pages) in &peer_bases {
        assert_eq!(
            base.as_ref(),
            Some(rep),
            "peer leaf {br:?} must delta-seal over representative {rep:?}, got {base:?}"
        );
        assert_eq!(
            *pages, 0,
            "identical peer leaf {br:?} must emit 0 delta pages"
        );
    }
    assert_eq!(peer_pages, 0);

    // Reopen C2 and verify exact resolution on every leaf.
    let reopened2 = reopen_named(request, ck2, src2, &medium, u64::MAX / 4);
    for leaf in &leaves {
        let obs = observe_branch(&reopened2, leaf);
        reopened2
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
                assert_eq!(
                    view.resolve(
                        PersistentNameKey::Entity(post_main_n[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                    Err(PersistentNameResolutionDenial::UnknownName {
                        name: PersistentNameKey::Entity(post_main_n[0]),
                    })
                );
                assert_eq!(
                    view.resolve(
                        PersistentNameKey::Entity(post_main_n[59]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                    Err(PersistentNameResolutionDenial::UnknownName {
                        name: PersistentNameKey::Entity(post_main_n[59]),
                    })
                );
            })
            .unwrap();
    }
}
