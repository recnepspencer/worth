use std::{collections::BTreeMap, num::NonZeroU64, path::Path};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BlobReadLimits;
use worth_store_physical_format::BlobGenerationPublicationV1;

use super::{
    super::{
        blob_crash::SCOPE_KEY,
        blob_ingest_process::observe_closed_store_with_limits,
        fixture::{admitted_blob_scope, serving_from_open},
    },
    child::{CHUNK_BYTES, TOTAL_BYTES},
};

/// Recomputes the child specification by absolute byte position, not from the
/// child's source buffer or a full-object expected allocation.
fn expected_byte(position: u64) -> u8 {
    (position.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (position >> 17)
        ^ (position >> 18).wrapping_mul(37)) as u8
}

pub(super) fn assert_fresh_stream(
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
    publication: BlobGenerationPublicationV1,
) {
    let serving = serving_from_open(root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(8192).unwrap());
    let published = blobs
        .resolve_publication(object, 1, &scope, limits)
        .expect("fresh C5 selected publication must resolve");
    assert_eq!(published.session().bytes(), session);
    let mut read = blobs
        .read(published, &scope, 0, TOTAL_BYTES as u64, limits)
        .expect("fresh reader must traverse the selected two-level tree");
    let mut actual = [0_u8; CHUNK_BYTES];
    let mut expected = [0_u8; CHUNK_BYTES];
    let mut actual_digest = Sha256::new();
    let mut expected_digest = Sha256::new();
    let mut position = 0_u64;
    loop {
        let used = read.read_next(&mut actual).unwrap();
        if used == 0 {
            break;
        }
        for (offset, byte) in expected[..used].iter_mut().enumerate() {
            *byte = expected_byte(position + offset as u64);
        }
        assert_eq!(
            &actual[..used],
            &expected[..used],
            "fresh streaming mismatch at byte {position}"
        );
        actual_digest.update(&actual[..used]);
        expected_digest.update(&expected[..used]);
        position += used as u64;
    }
    assert_eq!(position, TOTAL_BYTES as u64);
    assert_eq!(
        actual_digest.finalize().as_slice(),
        publication.logical_digest()
    );
    assert_eq!(
        expected_digest.finalize().as_slice(),
        publication.logical_digest()
    );
    drop(read);
    drop(blobs);
    serving.close();
}

pub(super) fn assert_independent_offline(root: &Path) {
    // The selected Store has more than 39,000 namespace files before the
    // interior-root marker. Keep this scheduled offline walk finite without
    // mistaking the smaller ordinary-journey budget for a completeness proof.
    let report = observe_closed_store_with_limits(
        root,
        "c11-blob-resume",
        "selected-interior-root",
        131_072,
        8 * 1024 * 1024 * 1024,
        32 * 1024 * 1024,
        1_800_000,
    );
    if report["completeness"] != "complete" {
        let rows = report["artifacts"].as_array().unwrap();
        let mut reasons = BTreeMap::new();
        for row in rows {
            if row["outcome"]["posture"] == "indeterminate" {
                *reasons
                    .entry(row["outcome"]["reason"].to_string())
                    .or_insert(0_usize) += 1;
            }
        }
        let examples = rows
            .iter()
            .filter(|row| row["outcome"]["posture"] != "intact")
            .take(3)
            .map(|row| (&row["family"], &row["identity"], &row["outcome"]))
            .collect::<Vec<_>>();
        panic!(
            "offline observer incomplete: completeness={}, consumed={}, limits={}, indeterminate_reasons={reasons:?}, examples={examples:?}",
            report["completeness"],
            report["consumed"],
            report["declared_limits"]
        );
    }
    let artifacts = report["artifacts"].as_array().unwrap();
    for (family, count) in [
        ("blob_resume_session", 65),
        ("blob_chunk_frame", 4097),
        ("blob_tree_node", 3),
        ("blob_generation_publication", 1),
    ] {
        let matching = artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            count,
            "{family} count differs; consumed={}",
            report["consumed"]
        );
        assert!(
            matching
                .iter()
                .all(|row| row["outcome"]["posture"] == "intact"),
            "{family} has a non-intact artifact: {:?}",
            matching
                .iter()
                .find(|row| row["outcome"]["posture"] != "intact")
                .map(|row| (&row["identity"], &row["outcome"]))
        );
    }
}
