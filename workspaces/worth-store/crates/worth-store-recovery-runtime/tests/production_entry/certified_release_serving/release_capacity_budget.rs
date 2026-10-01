//! The first V3 release reserves its complete recovery closure before any media effect.

use super::*;
use std::{fs, num::NonZeroU64, path::PathBuf};
use worth_store::physical_runtime::{
    BlobReclaimFailure, PhysicalOperationAllocationScope, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryRejoinResidentDenial,
};
use worth_store_physical_format::{
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, BLOB_CONTROL_FRAME_MAX_BYTES,
};

#[test]
fn one_head_recovery_scope_rejects_before_manifest_wal_or_root_effect() {
    std::thread::Builder::new()
        .name("release-capacity-low-recovery".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let limit = numeric_one_head_closure();
            let (world, proof, publication_record) =
                release_reopen::published_world::create(false, None, NonZeroU64::new(limit));
            let serving = world.serving();
            let policy = serving.residency_observation().admitted_policy();
            assert_eq!(policy.operation_bytes(), 32 << 20);
            assert_eq!(
                policy.scope_bytes(PhysicalOperationAllocationScope::Recovery),
                limit,
            );
            let before = snapshot_family(world.root());
            let counters = serving.media_counters();
            let selected = serving
                .certification_selected_latest_blob_publication()
                .unwrap()
                .expect("lawfully published source remains selected");
            let result = serving
                .blobs()
                .unwrap()
                .reclaim(release_request(proof, world.placement()))
                .and_then(|handle| handle.wait());
            match result {
                Err(BlobReclaimFailure::ReleaseCertificateBacking(
                    PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                )) => {
                    assert_eq!(admitted, limit);
                    assert!(required > admitted);
                }
                Err(other) => panic!("pre-effect release capacity denial: {other:?}"),
                Ok(_) => panic!("release unexpectedly completed at numeric-only closure"),
            }
            assert_eq!(snapshot_family(world.root()), before);
            let after = serving.media_counters();
            assert_eq!(after.append_attempts(), counters.append_attempts());
            assert_eq!(after.replacements(), counters.replacements());
            assert_eq!(after.file_creates(), counters.file_creates());
            assert_eq!(
                serving
                    .certification_selected_latest_blob_publication()
                    .unwrap()
                    .unwrap()
                    .record(),
                selected.record(),
            );
            assert_eq!(selected.record(), publication_record);
            world.close();
        })
        .expect("release capacity worker")
        .join()
        .expect("release capacity worker did not panic");
}

#[test]
fn sufficient_recovery_scope_drops_then_checkpoints_and_reopens_with_c8_seal() {
    std::thread::Builder::new()
        .name("release-capacity-sufficient-recovery".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, proof, record) =
                release_reopen::published_world::create(false, None, NonZeroU64::new(32 << 20));
            assert_eq!(
                world
                    .serving()
                    .residency_observation()
                    .admitted_policy()
                    .scope_bytes(PhysicalOperationAllocationScope::Recovery),
                32 << 20,
            );
            let receipt = world
                .serving()
                .blobs()
                .unwrap()
                .reclaim(release_request(proof, world.placement()))
                .expect("sufficient Recovery scope admits release")
                .wait()
                .expect("sufficient Recovery scope completes release");
            assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
            assert_eq!(receipt.remaining_payload_records(), 0);
            assert!(receipt.dropped_records().contains(&record));
            super::run_world((world, receipt, record), super::Mutation::None);
        })
        .expect("release capacity worker")
        .join()
        .expect("release capacity worker did not panic");
}

fn numeric_one_head_closure() -> u64 {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let page = u64::from(format.page_size().bytes());
    let entry = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let pair = std::mem::size_of::<(ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadEntryV1)>() as u64;
    // Current numeric admission for a projected first head: four control
    // maxima, three resident entry copies, four map pairs, 18 retained tree
    // nodes, and two 52-page transition windows. The new mandatory backing
    // charge must make equality with this older numeric subtotal insufficient.
    4 * BLOB_CONTROL_FRAME_MAX_BYTES as u64
        + 3 * entry
        + 4 * pair
        + 18 * page
        + 2 * 52 * page
        + ReleaseCustodyHeadEntryV1::ENCODED_BYTES as u64
}

fn release_request(
    proof: AdmittedBlobReleaseProof,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) -> BlobReclaimRequest<'static> {
    BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(1024).unwrap(),
        )
        .unwrap(),
    )
}

fn snapshot_family(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(directory).expect("qualified Store family") {
            let entry = entry.expect("qualified family member");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut files = Vec::new();
    visit(root, &root.join("families"), &mut files);
    files.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    files
}
