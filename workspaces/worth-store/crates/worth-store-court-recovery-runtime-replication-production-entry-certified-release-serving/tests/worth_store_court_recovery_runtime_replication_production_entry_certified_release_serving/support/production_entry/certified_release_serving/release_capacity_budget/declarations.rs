//! The first V3 release reserves its complete recovery closure before any media effect.

pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::path::Path, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run_world};
pub(crate) use {std::fs, std::num::NonZeroU64, std::path::PathBuf};
pub(crate) use {worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial};
pub(crate) use {worth_store_physical_format::ReleaseCustodyHeadEntryV1, worth_store_physical_format::ReleaseCustodyHeadKeyV1, worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES, worth_store_physical_format::CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES, worth_store_physical_format::MAX_CHECKPOINT_BINDING_RECORD_BYTES};

pub(crate) fn numeric_one_head_closure() -> u64 {
    let format = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap();
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
        + worth_store_physical_format::ReleaseCustodyHeadEntryV1::ENCODED_BYTES as u64
}

pub(crate) fn funded_one_head_limit() -> u64 {
    // Capture may retain two framed binding commands. Residency phase 1 raised
    // their wire maximum to cover every returned identity in a u16-sized batch.
    // Keep 4 MiB separately for the policy's roughly 2.3 MiB pin scan and capture
    // metadata/headroom; this is an ample fixture, not an exact admission quote.
    let command =
        MAX_CHECKPOINT_BINDING_RECORD_BYTES + CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES;
    numeric_one_head_closure() + 2 * command as u64 + (4 << 20)
}

pub(crate) fn release_request(
    proof: AdmittedBlobReleaseProof,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) -> BlobReclaimRequest<'static> {
    worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        placement,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1024).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn snapshot_family(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(directory).expect("qualified Store family") {
            let entry = entry.expect("qualified family member");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut files = Vec::new();
    visit(root, &root.join("families"), &mut files);
    files.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    files
}