//! Law-based property suite for `worth-store-snapshots`:
//! arbitrary snapshot publication, point-in-time read admission, same-ID
//! republication isolation, cross-snapshot rejection, over-broad read denial,
//! and bundle forgery refusal.
use proptest::prelude::*;
use worth_store_contracts::{DurableArtifactFamilyId, StableArtifactId};
use worth_store_snapshots::{
    reject_snapshot_bundle_layout_authority, snapshot_semantic_authority, PublishedSnapshotHandle,
    SnapshotId, SnapshotImageBundle, SnapshotLayoutAccessDenialKind, SnapshotReadRequest,
};

#[derive(Debug, Clone)]
enum SnapshotOp {
    Publish {
        id_num: u16,
        digest_tag: u32,
        declared_pages: u32,
    },
    RepublishSameId {
        target_idx: usize,
        digest_xor: u32,
        new_declared_pages: u32,
    },
    ReadExisting {
        target_idx: usize,
        requested_pages: u32,
    },
    ReadMismatched {
        handle_idx: usize,
        other_id_num: u16,
        requested_pages: u32,
    },
    ProbeBundleForgery {
        id_num: u16,
        digest_tag: u32,
        declared_pages: u32,
    },
}

fn page_count_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        3 => 0_u32..=32,
        2 => prop::sample::select(vec![
            0_u32,
            1,
            u16::MAX as u32 - 1,
            u16::MAX as u32,
            u16::MAX as u32 + 1,
            u32::MAX,
        ]),
    ]
}

fn snapshot_op_strategy() -> impl Strategy<Value = SnapshotOp> {
    prop_oneof![
        3 => (1_u16..=64, any::<u32>(), page_count_strategy())
            .prop_map(|(id_num, digest_tag, declared_pages)| SnapshotOp::Publish {
                id_num,
                digest_tag,
                declared_pages,
            }),
        2 => (any::<usize>(), 1_u32..=u32::MAX, page_count_strategy())
            .prop_map(|(target_idx, digest_xor, new_declared_pages)| {
                SnapshotOp::RepublishSameId {
                    target_idx,
                    digest_xor,
                    new_declared_pages,
                }
            }),
        4 => (any::<usize>(), page_count_strategy())
            .prop_map(|(target_idx, requested_pages)| SnapshotOp::ReadExisting {
                target_idx,
                requested_pages,
            }),
        3 => (any::<usize>(), 1_u16..=64, page_count_strategy())
            .prop_map(|(handle_idx, other_id_num, requested_pages)| SnapshotOp::ReadMismatched {
                handle_idx,
                other_id_num,
                requested_pages,
            }),
        2 => (1_u16..=64, any::<u32>(), page_count_strategy())
            .prop_map(|(id_num, digest_tag, declared_pages)| SnapshotOp::ProbeBundleForgery {
                id_num,
                digest_tag,
                declared_pages,
            }),
    ]
}

fn make_snapshot_id(id_num: u16) -> SnapshotId {
    let artifact = StableArtifactId::new(format!("snap-{id_num:04}")).expect("valid artifact id");
    SnapshotId::from_artifact_id(artifact)
}

fn assert_handle_bounds(
    handle: &PublishedSnapshotHandle,
    orig_id: &SnapshotId,
    orig_digest: &str,
    orig_declared: u32,
    requested_pages: u32,
) -> Result<(), TestCaseError> {
    let req = SnapshotReadRequest::new(orig_id.clone(), requested_pages);
    let res = handle.admit_layout_support(&req);
    if requested_pages <= orig_declared {
        let report = res.expect("bounded snapshot read must admit");
        prop_assert_eq!(
            report.family_id(),
            DurableArtifactFamilyId::PublicationSnapshotImage
        );
        prop_assert_eq!(report.snapshot_id(), orig_id);
        prop_assert_eq!(report.image_digest(), orig_digest);
        prop_assert_eq!(report.declared_page_count(), orig_declared);
        prop_assert_eq!(report.requested_page_count(), requested_pages);
        let est = report.support_estimate();
        prop_assert_eq!(
            est.planned_page_touches(),
            requested_pages.min(u16::MAX as u32) as u16
        );
        prop_assert_eq!(
            est.planned_publications(),
            if orig_declared == 0 { 0 } else { 1 }
        );
        prop_assert_eq!(est.planned_maintenance_reads(), 1);
    } else {
        prop_assert_eq!(
            res.unwrap_err().kind(),
            SnapshotLayoutAccessDenialKind::SnapshotReadBroadensBeyondPublishedImage
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn snapshot_publications_preserve_point_in_time_isolation_and_read_bounds(
        ops in prop::collection::vec(snapshot_op_strategy(), 1..=48),
    ) {
        let authority = snapshot_semantic_authority();
        let mut published: Vec<(PublishedSnapshotHandle, SnapshotId, u32, String, u32)> =
            Vec::new();

        for op in ops {
            match op {
                SnapshotOp::Publish {
                    id_num,
                    digest_tag,
                    declared_pages,
                } => {
                    let id = make_snapshot_id(id_num);
                    let digest = format!("sha256:{digest_tag:08x}");
                    let handle = authority.publish_snapshot_image(
                        id.clone(),
                        digest.clone(),
                        declared_pages,
                    );
                    prop_assert_eq!(handle.snapshot_id(), &id);
                    prop_assert_eq!(handle.image_digest(), digest.as_str());
                    prop_assert_eq!(handle.declared_page_count(), declared_pages);
                    published.push((handle, id, digest_tag, digest, declared_pages));
                }
                SnapshotOp::RepublishSameId {
                    target_idx,
                    digest_xor,
                    new_declared_pages,
                } => {
                    if published.is_empty() {
                        continue;
                    }
                    let pos = target_idx % published.len();
                    let (_, orig_id, orig_tag, orig_digest, orig_declared) = (
                        &published[pos].0,
                        published[pos].1.clone(),
                        published[pos].2,
                        published[pos].3.clone(),
                        published[pos].4,
                    );
                    let new_tag = orig_tag ^ digest_xor;
                    let new_digest = format!("sha256:{new_tag:08x}");
                    let new_declared = if new_declared_pages == orig_declared {
                        orig_declared.wrapping_add(1)
                    } else {
                        new_declared_pages
                    };
                    let new_handle = authority.publish_snapshot_image(
                        orig_id.clone(),
                        new_digest.clone(),
                        new_declared,
                    );
                    // Prove both handles retain their own distinct metadata and read bounds.
                    let orig_handle = &published[pos].0;
                    assert_handle_bounds(orig_handle, &orig_id, &orig_digest, orig_declared, orig_declared)?;
                    if let Some(over) = orig_declared.checked_add(1) {
                        assert_handle_bounds(orig_handle, &orig_id, &orig_digest, orig_declared, over)?;
                    }
                    assert_handle_bounds(&new_handle, &orig_id, &new_digest, new_declared, new_declared)?;
                    if let Some(over) = new_declared.checked_add(1) {
                        assert_handle_bounds(&new_handle, &orig_id, &new_digest, new_declared, over)?;
                    }
                    published.push((new_handle, orig_id, new_tag, new_digest, new_declared));
                }
                SnapshotOp::ReadExisting {
                    target_idx,
                    requested_pages,
                } => {
                    if published.is_empty() {
                        continue;
                    }
                    let (handle, orig_id, _, orig_digest, orig_declared) =
                        &published[target_idx % published.len()];
                    assert_handle_bounds(
                        handle,
                        orig_id,
                        orig_digest,
                        *orig_declared,
                        requested_pages,
                    )?;
                }
                SnapshotOp::ReadMismatched {
                    handle_idx,
                    other_id_num,
                    requested_pages,
                } => {
                    if published.is_empty() {
                        continue;
                    }
                    let (handle, orig_id, _, orig_digest, orig_declared) =
                        &published[handle_idx % published.len()];
                    let req_id = make_snapshot_id(other_id_num);
                    if &req_id != orig_id {
                        let req = SnapshotReadRequest::new(req_id, requested_pages);
                        prop_assert_eq!(
                            handle.admit_layout_support(&req).unwrap_err().kind(),
                            SnapshotLayoutAccessDenialKind::SnapshotHandleDoesNotMatchReadRequest
                        );
                    } else {
                        assert_handle_bounds(
                            handle,
                            orig_id,
                            orig_digest,
                            *orig_declared,
                            requested_pages,
                        )?;
                    }
                }
                SnapshotOp::ProbeBundleForgery {
                    id_num,
                    digest_tag,
                    declared_pages,
                } => {
                    let id = make_snapshot_id(id_num);
                    let digest = format!("sha256:{digest_tag:08x}");
                    let bundle = SnapshotImageBundle::new(id, digest, declared_pages);
                    let denial = reject_snapshot_bundle_layout_authority(&bundle).unwrap_err();
                    prop_assert_eq!(
                        denial.kind(),
                        SnapshotLayoutAccessDenialKind::SnapshotBundleCannotStandInForLayoutAuthority
                    );
                }
            }
        }

        // Verify every published handle still agrees with its exact creation-time metadata
        // even after arbitrary subsequent publications and same-ID republications.
        for (handle, orig_id, _, orig_digest, orig_declared) in &published {
            assert_handle_bounds(handle, orig_id, orig_digest, *orig_declared, *orig_declared)?;
        }
    }
}
