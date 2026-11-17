use sha2::{Digest, Sha256};

use super::{
    OfflineMediaClosureEntry, OfflineMediaConsistencyBasis, OfflineMediaReadDenial,
    ReadOnlyOfflineMediaCapability,
};

#[test]
fn consumed_path_vector_is_included_in_peak_allocation_admission() {
    let directory = tempfile::tempdir().expect("directory");
    // Spare consumed vector capacity must be admitted even when later file
    // bookkeeping would otherwise dominate the peak on this platform.
    let mut paths = Vec::with_capacity(4096);
    let mut entries = Vec::new();
    for index in 0..16 {
        let path = directory.path().join(format!(
            "artifact-{index:04}-{}.media",
            "long-path-payload".repeat(8)
        ));
        let bytes = vec![index as u8; 5];
        std::fs::write(&path, &bytes).expect("media");
        entries.push(
            OfflineMediaClosureEntry::new(&path, bytes.len() as u64, Sha256::digest(&bytes).into())
                .expect("closure entry"),
        );
        paths.push(path);
    }
    let basis = OfflineMediaConsistencyBasis::content_addressed_closure_from_owned_entries(
        "owned-input-accounting",
        entries,
    )
    .expect("basis");
    let owned = ReadOnlyOfflineMediaCapability::open_bounded_from_owned_paths(
        {
            let mut input = Vec::with_capacity(paths.capacity());
            input.extend(paths.iter().cloned());
            input
        },
        basis.clone(),
        u64::MAX,
    )
    .expect("owned path input");
    let lazy = ReadOnlyOfflineMediaCapability::open_bounded(
        paths.iter().cloned(),
        basis.clone(),
        u64::MAX,
    )
    .expect("caller-retained lazy input");
    assert!(owned.peak_owned_allocation_bytes() > lazy.peak_owned_allocation_bytes());
    let exact_peak = owned.peak_owned_allocation_bytes();

    let denial =
        ReadOnlyOfflineMediaCapability::open_bounded_from_owned_paths(paths, basis, exact_peak - 1)
            .expect_err("one byte below the consumed-input peak must deny");
    assert!(matches!(
        denial,
        OfflineMediaReadDenial::OwnedAllocationBudgetExceeded {
            admitted,
            limit,
        } if admitted == exact_peak && limit == exact_peak - 1
    ));
}

#[test]
fn missing_media_is_a_path_localized_io_denial() {
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("missing.page");
    let basis = closure(&path, b"expected-owner-bytes");

    let denial = ReadOnlyOfflineMediaCapability::open_bounded([path.clone()], basis, 4096)
        .expect_err("missing physical media cannot become an offline capability");

    assert!(matches!(
        denial,
        OfflineMediaReadDenial::Io { path: denied, source }
            if denied == path && source.kind() == std::io::ErrorKind::NotFound
    ));
}

#[test]
fn directory_substitution_is_not_admitted_as_an_offline_file() {
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("substituted.page");
    std::fs::create_dir(&path).expect("directory substitution");
    let basis = closure(&path, b"expected-owner-bytes");

    assert!(matches!(
        ReadOnlyOfflineMediaCapability::open_bounded([path.clone()], basis, 4096),
        Err(OfflineMediaReadDenial::NotAFile { path: denied }) if denied == path
    ));
}

#[cfg(windows)]
#[test]
fn operating_system_read_denial_remains_typed_and_path_localized() {
    use std::os::windows::fs::OpenOptionsExt;

    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("sharing-denied.page");
    let bytes = b"owner-media";
    std::fs::write(&path, bytes).expect("media");
    let basis = closure(&path, bytes);
    let _exclusive = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .expect("exclusive read handle");

    let denial = ReadOnlyOfflineMediaCapability::open_bounded([path.clone()], basis, 4096)
        .expect_err("OS-denied media cannot become an offline capability");

    assert!(matches!(
        denial,
        OfflineMediaReadDenial::Io { path: denied, source }
            if denied == path && matches!(source.raw_os_error(), Some(5) | Some(32))
    ));
}

fn closure(path: &std::path::Path, bytes: &[u8]) -> OfflineMediaConsistencyBasis {
    OfflineMediaConsistencyBasis::content_addressed_closure(
        "offline-capability-denial",
        [
            OfflineMediaClosureEntry::new(path, bytes.len() as u64, Sha256::digest(bytes).into())
                .expect("closure entry"),
        ],
    )
    .expect("content closure")
}
