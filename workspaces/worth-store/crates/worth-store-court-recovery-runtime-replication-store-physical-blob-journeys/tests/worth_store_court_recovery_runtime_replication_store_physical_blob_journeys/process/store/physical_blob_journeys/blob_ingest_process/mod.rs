#[allow(unused_imports)] use crate::physical_blob_journeys::blob_ingest_process::*;
use {std::num::NonZeroU64, std::path::Path, std::process::Command};

#[cfg(not(all()))]
use {sha2::Digest, sha2::Sha256};
use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline};
use {worth_store_blob_chunks::BlobChunkSize};

use {worth_store_testkit_store::allocation_probe::physical_blob_journeys::peak_live_bytes_during};
use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};


pub(super) use {crate::physical_blob_journeys::blob_ingest_process::range_read::reader};

// The heap ceiling measures the production feature graph. Certification's
// retained allocation trace is separately exercised by the crash/trace lane.
#[cfg(not(all()))]
#[test]
fn eight_mib_blob_survives_fresh_reopen_and_independent_offline_observation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().to_path_buf();
    let negative_output = run_child("allocator_negative", &root, None);
    let negative_peak: usize = marked_line(&negative_output, NEGATIVE_PREFIX)
        .parse()
        .unwrap();
    assert!(
        negative_peak > SOURCE_WINDOW + 5 * 1024 * 1024,
        "whole-object negative control unexpectedly fit: {negative_peak}"
    );
    let writer_output = run_child("writer", &root, None);
    let writer_process = marked_line(&writer_output, PROCESS_PREFIX);
    let publication = marked_line(&writer_output, PUBLICATION_PREFIX);
    let reader_output = run_child("reader", &root, Some(publication));
    let reader_process = marked_line(&reader_output, PROCESS_PREFIX);
    assert_ne!(writer_process, reader_process);
    assert_ne!(writer_process, std::process::id().to_string());
    assert_ne!(reader_process, std::process::id().to_string());
    let observed_digest = marked_line(&reader_output, READ_PREFIX);
    assert_eq!(observed_digest, expected_range_digest());

    let report = observe_closed_store(&root);
    assert_ne!(report["process"], writer_process);
    assert_ne!(report["process"], reader_process);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    for (family, expected) in [
        ("blob_resume_session", 1),
        ("blob_chunk_frame", OBJECT_BYTES / CHUNK_BYTES),
        ("blob_tree_node", 1),
        ("blob_generation_publication", 1),
    ] {
        let members = artifacts
            .iter()
            .filter(|artifact| artifact["family"] == family)
            .collect::<Vec<_>>();
        assert_eq!(members.len(), expected, "{family}: {report}");
        for member in members {
            assert_eq!(member["outcome"]["posture"], "intact", "{member}");
            assert!(
                member["range"].is_null(),
                "blob record is a routed composite: {member}"
            );
        }
    }
    let consumed_bytes = report["consumed"]["bytes"].as_u64().unwrap();
    assert!(
        consumed_bytes >= OBJECT_BYTES as u64,
        "observer skipped blob bytes"
    );
    assert!(
        consumed_bytes <= 256 * SOURCE_WINDOW as u64,
        "observer exceeded byte budget"
    );
}

#[cfg(not(all()))]
fn run_child(role: &str, root: &Path, publication: Option<&str>) -> String {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "c11_blob_child_role",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE_ENV, role)
        .env(ROOT_ENV, root);
    if let Some(publication) = publication {
        command.env(PUBLICATION_ENV, publication);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "C11 blob {role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[cfg(not(all()))]
pub(super) fn observe_closed_store(root: &Path) -> serde_json::Value {
    observe_closed_store_named(root, "c11-blob-8mib", "fresh-reopen-and-offline-observe")
}

#[cfg(not(all()))]
fn marked_line<'a>(stdout: &'a str, prefix: &str) -> &'a str {
    stdout
        .lines()
        // libtest may print the test name and the first child line together.
        .find_map(|line| line.split_once(prefix).map(|(_, value)| value))
        .unwrap_or_else(|| panic!("missing {prefix:?} in child output: {stdout}"))
}

#[cfg(not(all()))]
fn expected_range_digest() -> String {
    let mut digest = sha2::Sha256::new();
    let mut source = vec![0_u8; 64 * 1024];
    for offset in (READ_START..READ_END).step_by(source.len()) {
        let len = source.len().min(READ_END - offset);
        fill_expected(offset, &mut source[..len]);
        digest.update(&source[..len]);
    }
    hex(&digest.finalize())
}
