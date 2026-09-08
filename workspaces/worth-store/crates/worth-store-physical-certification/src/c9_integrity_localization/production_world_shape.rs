use std::collections::BTreeSet;
use std::path::Path;

use super::production_profile::ProductionWorldProfile;
use super::ClosedStoreProcessManifest;

pub(super) fn require_shape(
    root: &Path,
    manifest: &ClosedStoreProcessManifest,
    profile: ProductionWorldProfile,
) {
    let mut segments = 0;
    let mut extents = 0;
    let mut wal_segments = 0;
    let mut page_frames = 0;
    let mut root_branch = false;
    let mut segment_branch = false;
    let mut free_space_branch = false;
    let mut checkpoint_kinds = BTreeSet::new();
    for path in manifest.paths() {
        let bytes = std::fs::read(root.join(path)).expect("read production shape");
        let name = path.file_name().unwrap().to_string_lossy();
        if name.ends_with(".pages") {
            segments += 1;
            page_frames += bytes
                .chunks(profile.page_size().bytes() as usize)
                .filter(|frame| frame.starts_with(b"WRC5FRM\0") && frame[8] == 3)
                .count();
        }
        if name.ends_with(".data") && name.starts_with("extent-") {
            extents += 1;
        }
        if bytes.starts_with(b"WORTHWAL") {
            wal_segments += 1;
        }
        if bytes.starts_with(b"WRC5FRM\0") && bytes.len() > 88 {
            let branch = bytes[68] == 2;
            match bytes[8] {
                8 => root_branch |= branch,
                9 => segment_branch |= branch,
                10 => free_space_branch |= branch,
                _ => {}
            }
        }
        if bytes.starts_with(b"WCP7REC\0") {
            let mut offset = 0;
            while offset < bytes.len() {
                assert_eq!(&bytes[offset..offset + 8], b"WCP7REC\0");
                checkpoint_kinds.insert(bytes[offset + 9]);
                let payload =
                    u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap())
                        as usize;
                offset += 20 + payload;
            }
            assert_eq!(offset, bytes.len());
        }
    }
    assert!(
        page_frames >= 2,
        "two separately scoped inline frames are required"
    );
    assert!(
        manifest
            .paths()
            .any(|path| path.ends_with("root-previous.selector")),
        "retainable previous selector required"
    );
    if profile == ProductionWorldProfile::Primary16KiB {
        assert!(segments >= 3, "three production data segments required");
        assert!(extents >= 1, "production extent required");
        assert!(
            wal_segments >= 2,
            "two production WAL segments required: {wal_segments}"
        );
        assert!(
            root_branch && segment_branch,
            "multi-level root and membership trees required"
        );
        assert_eq!(
            checkpoint_kinds,
            BTreeSet::from([1, 2, 3, 4, 5]),
            "all five production checkpoint record families required"
        );
    }
    println!("C9 topology profile={} segments={segments} pages={page_frames} extents={extents} wal_segments={wal_segments} root_branch={root_branch} segment_branch={segment_branch} free_space_branch={free_space_branch} checkpoint_kinds={checkpoint_kinds:?}", profile.label());
}
