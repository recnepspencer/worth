use {
    worth_store_snapshots::reject_snapshot_bundle_layout_authority,
    worth_store_snapshots::snapshot_semantic_authority,
};

#[test]
fn snapshot_family_binds_snapshot_reads_to_admitted_image_authority() {
    let snapshot_id = worth_store_snapshots::SnapshotId::from_artifact_id(
        worth_store_contracts::StableArtifactId::new("phase23-snapshot").unwrap(),
    );
    let handle = snapshot_semantic_authority().publish_snapshot_image(
        snapshot_id.clone(),
        "sha256:image",
        8,
    );
    let request = worth_store_snapshots::SnapshotReadRequest::new(snapshot_id.clone(), 5);

    let report = handle
        .admit_layout_support(&request)
        .expect("admitted snapshot image");
    assert_eq!(
        report.family_id(),
        worth_store_contracts::DurableArtifactFamilyId::PublicationSnapshotImage
    );
    assert_eq!(report.snapshot_id(), &snapshot_id);
    assert_eq!(report.image_digest(), "sha256:image");
    assert_eq!(report.declared_page_count(), 8);
    assert_eq!(report.requested_page_count(), 5);
    assert_eq!(report.support_estimate().planned_publications(), 1);
    assert_eq!(report.support_estimate().planned_maintenance_reads(), 1);
    assert_eq!(report.support_estimate().planned_page_touches(), 5);

    let bundle =
        worth_store_snapshots::SnapshotImageBundle::new(snapshot_id.clone(), "sha256:image", 8);
    let denial = reject_snapshot_bundle_layout_authority(&bundle).unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_snapshots::SnapshotLayoutAccessDenialKind::SnapshotBundleCannotStandInForLayoutAuthority
    );

    let broad_request = worth_store_snapshots::SnapshotReadRequest::new(snapshot_id.clone(), 9);
    let denial = handle.admit_layout_support(&broad_request).unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_snapshots::SnapshotLayoutAccessDenialKind::SnapshotReadBroadensBeyondPublishedImage
    );

    let mismatched = worth_store_snapshots::SnapshotReadRequest::new(
        worth_store_snapshots::SnapshotId::from_artifact_id(
            worth_store_contracts::StableArtifactId::new("phase23-other-snapshot").unwrap(),
        ),
        4,
    );
    let denial = handle.admit_layout_support(&mismatched).unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_snapshots::SnapshotLayoutAccessDenialKind::SnapshotHandleDoesNotMatchReadRequest
    );
}
