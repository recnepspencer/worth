use {worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::PhysicalRetirementDenial};

use {crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::physical_blob_journeys::blob_reclaim::abandoned_prefix, crate::physical_blob_journeys::blob_reclaim::request, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

#[test]
fn selected_payload_drop_reopens_with_its_unretired_native_extent() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.reopen-native-retirement.scope");
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let token = abandoned_prefix(&serving, &scope);
    let frontier = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(frontier.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert_eq!(frontier.remaining_payload_records(), 2);

    serving.certification_owe_before_retirement_barrier();
    let payload = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(payload.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert_eq!(payload.dropped_records().len(), 1);
    assert_eq!(payload.displaced_extents().len(), 1);
    assert_eq!(
        payload.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(worth_store::physical_runtime::PhysicalRetirementDenial::Waiting)
    );
    assert_eq!(payload.bytes_released(), 0);
    let displaced_range = payload.displaced_extents()[0].range();
    let retained = serving.certification_charged_growth_bytes();
    serving.certification_release_owed_background_turn();
    serving.close();

    let reopened = serving_from_open(directory.path());
    assert_eq!(
        reopened.certification_charged_growth_bytes(),
        retained,
        "fresh owner install must reconstruct the selected drop's native extent obligation"
    );
    reopened.retire_displaced_segment().unwrap();
    assert!(
        reopened.certification_charged_growth_bytes() < retained,
        "the exact displaced range of {} bytes must stop being charged",
        displaced_range.length()
    );
    assert_eq!(
        reopened.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "the selected drop creates one native retirement obligation"
    );
    reopened.close();

    let reopened = serving_from_open(directory.path());
    assert_eq!(
        reopened.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "the published free range must not be reconstructed as garbage twice"
    );
    reopened.close();
}
