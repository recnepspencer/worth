#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_occurrence_corruption::*;

use crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named;

#[test]
fn missing_occurrence_on_real_extent_fails_independent_offline_custody() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    std::fs::create_dir(&root).unwrap();
    write_two_byte_blob(&root);

    let before = observe_closed_store_named(&root, "c11-occurrence", "before-omission");
    assert_eq!(before["completeness"], "complete", "{before}");
    let before_chunk = blob_rows(&before, "blob_chunk_frame");
    assert_eq!(before_chunk.len(), 1, "{before}");
    assert_eq!(before_chunk[0]["outcome"]["posture"], "intact", "{before}");

    omit_occurrence_and_reseal_inner_and_outer_frames(&root);

    let after = observe_closed_store_named(&root, "c11-occurrence", "after-omission");
    assert_eq!(after["completeness"], "complete", "{after}");
    let after_chunk = blob_rows(&after, "blob_chunk_frame");
    assert_eq!(after_chunk.len(), 1, "{after}");
    assert_eq!(after_chunk[0]["outcome"]["posture"], "damaged", "{after}");
    assert_eq!(
        after_chunk[0]["outcome"]["cause"], "malformed_payload",
        "custody omission must not be attributed to a checksum or setup failure: {after}"
    );
    for family in ["blob_resume_session", "extent_chunk_frame"] {
        let rows = blob_rows(&after, family);
        assert!(!rows.is_empty(), "{family}: {after}");
        assert!(
            rows.iter().all(|row| row["outcome"]["posture"] == "intact"),
            "{family}: {after}"
        );
    }
}
