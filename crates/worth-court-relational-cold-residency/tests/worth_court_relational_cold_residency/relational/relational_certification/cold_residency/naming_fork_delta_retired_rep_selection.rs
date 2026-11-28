//! End-to-end courts for representative-leaf selection over retired C1
//! intermediates when:
//! 1. The lexicographically first peer has a raised naming retention floor,
//!    requiring an uncompacted peer to be chosen as representative.
//! 2. The retired C1 intermediate was itself materialized (`delta_base = None`)
//!    because its parent's retention floor was raised before C1, requiring the
//!    representative to seal once so all peers alias it instead of resealing
//!    the full table `B` times.

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
    create_named_entities, durable_named_runtime, observe_branch, reopen_named,
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
fn retired_intermediate_skips_high_retention_leaf_as_representative() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let base_keys: Vec<String> = (0..300).map(|i| format!("retrep-base-{i:03}")).collect();
    let (c_base, base_e, base_n) = create_named_entities(request, &producer, &main, &base_keys);
    let fork_commit = c_base.commit.commit_id;

    let mid = BranchId("retrep-shared-mid".into());
    fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
    let mid_obs = observe_branch(&producer, &mid);
    let leaves: Vec<BranchId> = (0..12usize)
        .map(|i| BranchId(format!("retrep-leaf-{i:02}")))
        .collect();
    for leaf in &leaves {
        fork_retained_at(request, &producer, &mid_obs, leaf).unwrap();
    }
    let post_main_keys: Vec<String> = (0..60).map(|i| format!("retrep-post-{i:03}")).collect();
    let (_, _, post_main_n) = create_named_entities(request, &producer, &main, &post_main_keys);

    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let mut reopened1 = reopen_named(request, ck1, src1.clone(), &medium, u64::MAX / 4);
    // Raise ONLY the lexicographically first leaf's retention floor.
    reopened1
        .advance_naming_retention_floor(&leaves[0], fork_commit)
        .unwrap();
    let mid_id = reopened1.branch_identity(&mid).unwrap();
    assert!(reopened1
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let (ck2, src2) = capture_external_carrying(request, &reopened1, &src1);
    let body2 = ck2.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming2 = field(field(body2, "manifest"), "naming");
    let mut over_main = Vec::new();
    let mut peers = Vec::new();
    for row in rows(field(naming2, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if !br.0.starts_with("retrep-leaf-") {
            continue;
        }
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        let page_count = rows(field(row, "pages")).len();
        if base.as_ref() == Some(&main) {
            over_main.push(br);
        } else {
            peers.push((br, base, page_count));
        }
    }
    assert_eq!(
        over_main,
        vec![leaves[1].clone()],
        "must pick uncompacted leaf-01 over high-floor leaf-00 as sole main-rebased representative"
    );
    assert_eq!(peers.len(), 11);
    for (br, base, pages) in &peers {
        assert_eq!(
            base.as_ref(),
            Some(&leaves[1]),
            "peer {br:?} must alias uncompacted representative leaf-01, got {base:?}"
        );
        assert_eq!(*pages, 0, "peer {br:?} must emit 0 delta pages");
    }

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
            })
            .unwrap();
    }
}

#[test]
fn retired_materialized_c1_base_materializes_one_rep_and_aliases_peers() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let base_keys: Vec<String> = (0..300).map(|i| format!("matrep-base-{i:03}")).collect();
    let (_, base_e, base_n) = create_named_entities(request, &producer, &main, &base_keys);

    let mid = BranchId("matrep-shared-mid".into());
    fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
    let mid_obs = observe_branch(&producer, &mid);
    let leaves: Vec<BranchId> = (0..12usize)
        .map(|i| BranchId(format!("matrep-leaf-{i:02}")))
        .collect();
    for leaf in &leaves {
        fork_retained_at(request, &producer, &mid_obs, leaf).unwrap();
    }
    // Advance main's retention floor BEFORE C1 so `mid` must materialize in C1
    // (`delta_base = None`), while all 12 leaves alias `mid`.
    let (c_post, _, post_main_n) =
        create_named_entities(request, &producer, &main, &["matrep-main-post".into()]);
    producer
        .advance_naming_retention_floor(&main, c_post.commit.commit_id)
        .unwrap();

    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let body1 = ck1.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    for row in rows(field(field(field(body1, "manifest"), "naming"), "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        if br == mid {
            assert_eq!(
                base, None,
                "C1 mid must materialize under raised main floor"
            );
        } else if br.0.starts_with("matrep-leaf-") {
            assert_eq!(base.as_ref(), Some(&mid), "C1 leaf {br:?} must alias mid");
        }
    }

    let mut reopened1 = reopen_named(request, ck1, src1.clone(), &medium, u64::MAX / 4);
    // Explicitly record `Some(CommitId(0))` on the lexicographically first leaf
    // so `None` siblings strictly outrank `Some(0)` and remain peer-compatible.
    reopened1
        .advance_naming_retention_floor(&leaves[0], worth_relational::facade::history::CommitId(0))
        .unwrap();
    let mid_id = reopened1.branch_identity(&mid).unwrap();
    assert!(reopened1
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let (ck2, src2) = capture_external_carrying(request, &reopened1, &src1);
    let body2 = ck2.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming2 = field(field(body2, "manifest"), "naming");
    let mut materialized = Vec::new();
    let mut peers = Vec::new();
    for row in rows(field(naming2, "branches")) {
        let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if !br.0.starts_with("matrep-leaf-") {
            continue;
        }
        let base: Option<BranchId> = rmp_serde::from_slice(field(row, "delta_base")).unwrap();
        let page_count = rows(field(row, "pages")).len();
        if base.is_none() {
            materialized.push(br);
        } else {
            peers.push((br, base, page_count));
        }
    }
    assert_eq!(
        materialized,
        vec![leaves[1].clone()],
        "uncompacted leaf-01 (floor=None) must beat leaf-00 (floor=Some(0)) as sole materialized representative"
    );
    let rep = &materialized[0];
    assert_eq!(peers.len(), 11);
    for (br, base, pages) in &peers {
        assert_eq!(
            base.as_ref(),
            Some(rep),
            "peer {br:?} must alias materialized representative {rep:?}, got {base:?}"
        );
        assert_eq!(*pages, 0, "identical peer {br:?} must emit 0 delta pages");
    }

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
            })
            .unwrap();
    }
}
