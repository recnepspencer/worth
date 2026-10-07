use std::{
    num::NonZeroU64,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(not(feature = "certification-test-authority"))]
use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::allocation_probe::peak_live_bytes_during;
use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

#[path = "blob_ingest_process/range_read.rs"]
mod range_read;
pub(super) use range_read::reader;

const OBJECT_BYTES: usize = 8 * 1024 * 1024;
const SOURCE_WINDOW: usize = 1024 * 1024;
const CHUNK_BYTES: usize = 256 * 1024;
const READ_START: usize = 128 * 1024;
const READ_END: usize = 1_152 * 1024;
const SELECTED_SCAN_LIMIT: u64 = 512;
const SCOPE_KEY: &str = "c11.blob.process.scope";
#[cfg(not(feature = "certification-test-authority"))]
const ROLE_ENV: &str = "WORTH_STORE_C11_BLOB_CHILD_ROLE";
#[cfg(not(feature = "certification-test-authority"))]
const ROOT_ENV: &str = "WORTH_STORE_C11_BLOB_CHILD_ROOT";
const PUBLICATION_ENV: &str = "WORTH_STORE_C11_BLOB_PUBLICATION";
const PUBLICATION_PREFIX: &str = "C11_BLOB_PUBLICATION ";
const READ_PREFIX: &str = "C11_BLOB_READ ";
const PROCESS_PREFIX: &str = "C11_BLOB_PROCESS ";
const NEGATIVE_PREFIX: &str = "C11_BLOB_NEGATIVE_PEAK ";

// The heap ceiling measures the production feature graph. Certification's
// retained allocation trace is separately exercised by the crash/trace lane.
#[cfg(not(feature = "certification-test-authority"))]
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

pub(super) fn writer(root: &Path) {
    println!("{PROCESS_PREFIX}{}", std::process::id());
    let serving = serving_from_initialization(root);
    let limits = BlobReadLimits::new(NonZeroU64::new(SELECTED_SCAN_LIMIT).unwrap());
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        OBJECT_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(64).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let ((published, memory), measured_peak) = peak_live_bytes_during(|| {
        let mut session = blobs
            .begin_ingest(declaration, placement(), SOURCE_WINDOW as u64, limits)
            .unwrap();
        let mut source = vec![0_u8; SOURCE_WINDOW];
        for window in 0..OBJECT_BYTES / SOURCE_WINDOW {
            fill_expected(window * SOURCE_WINDOW, &mut source);
            session.push(&source).unwrap();
        }
        let memory = session.memory_observation();
        let published = session.finish().unwrap();
        (published, memory)
    });
    {
        assert_eq!(memory.admitted_window(), SOURCE_WINDOW as u64);
        assert_eq!(memory.ceiling(), (SOURCE_WINDOW + 5 * 1024 * 1024) as u64);
        assert!(memory.peak_charged_bytes() <= memory.ceiling());
    }
    assert!(
        measured_peak <= memory.ceiling() as usize,
        "process-observed heap peak {measured_peak} exceeded W+5MiB {}",
        memory.ceiling(),
    );
    println!(
        "C11_BLOB_MEMORY ledger_peak={} measured_peak={}",
        memory.peak_charged_bytes(),
        measured_peak
    );
    assert_eq!(published.generation().sequence(), 1);
    println!(
        "{PUBLICATION_PREFIX}{}:{}",
        hex(&published.object().bytes()),
        published.generation().sequence(),
    );
    serving.close();
}

pub(super) fn allocator_negative_control() {
    let (_, peak) = peak_live_bytes_during(|| {
        let whole = vec![0_u8; OBJECT_BYTES];
        std::hint::black_box(&whole);
    });
    println!("{NEGATIVE_PREFIX}{peak}");
}

#[cfg(not(feature = "certification-test-authority"))]
fn run_child(role: &str, root: &Path, publication: Option<&str>) -> String {
    let mut command = Command::new(std::env::current_exe().unwrap());
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

#[cfg(not(feature = "certification-test-authority"))]
pub(super) fn observe_closed_store(root: &Path) -> serde_json::Value {
    observe_closed_store_named(root, "c11-blob-8mib", "fresh-reopen-and-offline-observe")
}

pub(super) fn observe_closed_store_named(
    root: &Path,
    run: &str,
    scenario: &str,
) -> serde_json::Value {
    observe_closed_store_with_limits(root, run, scenario, 4096, 268_435_456, 2_097_152, 600_000)
}

/// A complete offline observation in which no artifact is damaged. Returns
/// the artifacts. It says nothing about an artifact the observer did not
/// reach: a caller that means "these rows are intact" asserts those rows.
pub(super) fn observed_without_damage(
    root: &Path,
    run: &str,
    scenario: &str,
) -> Vec<serde_json::Value> {
    let report = observe_closed_store_named(root, run, scenario);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap().clone();
    for artifact in &artifacts {
        assert_ne!(artifact["outcome"]["posture"], "damaged", "{artifact}");
    }
    artifacts
}

/// The root a cleanly closed store selects: the newest root manifest, then the
/// routing blocks of its generation. A superseded generation is no longer
/// addressed and observes unknown.
pub(super) fn selected_root(artifacts: &[serde_json::Value]) -> Vec<&serde_json::Value> {
    let of_family = |family: &'static str| {
        artifacts
            .iter()
            .filter(move |artifact| artifact["family"] == family)
    };
    let manifest = of_family("root_manifest")
        .max_by_key(|artifact| artifact["generation"].as_u64())
        .expect("a root manifest is observed");
    let routing_blocks = of_family("root_routing_block")
        .filter(|artifact| artifact["generation"] == manifest["generation"]);
    std::iter::once(manifest).chain(routing_blocks).collect()
}

pub(super) fn observe_closed_store_with_limits(
    root: &Path,
    run: &str,
    scenario: &str,
    maximum_entries: u64,
    maximum_bytes: u64,
    maximum_report_bytes: u64,
    maximum_elapsed_milliseconds: u64,
) -> serde_json::Value {
    let binary = std::env::var_os("WORTH_C9_OBSERVER_EXECUTABLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .and_then(Path::parent)
                .unwrap()
                .join(format!(
                    "physical_store_integrity_observer{}",
                    std::env::consts::EXE_SUFFIX
                ))
        });
    assert!(
        binary.is_file(),
        "standalone observer missing: {}",
        binary.display()
    );
    let maximum_open_files = if cfg!(windows) { "6" } else { "5" };
    let output = Command::new(&binary)
        .arg("observe")
        .arg("--store-root")
        .arg(root)
        .args([
            "--report",
            "-",
            "--max-entries",
            &maximum_entries.to_string(),
            "--max-bytes",
            &maximum_bytes.to_string(),
            "--max-open-files",
            maximum_open_files,
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            &maximum_elapsed_milliseconds.to_string(),
            "--max-report-bytes",
            &maximum_report_bytes.to_string(),
            "--run",
            run,
            "--scenario",
            scenario,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "standalone observer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[cfg(not(feature = "certification-test-authority"))]
fn marked_line<'a>(stdout: &'a str, prefix: &str) -> &'a str {
    stdout
        .lines()
        // libtest may print the test name and the first child line together.
        .find_map(|line| line.split_once(prefix).map(|(_, value)| value))
        .unwrap_or_else(|| panic!("missing {prefix:?} in child output: {stdout}"))
}

#[cfg(not(feature = "certification-test-authority"))]
fn expected_range_digest() -> String {
    let mut digest = Sha256::new();
    let mut source = vec![0_u8; 64 * 1024];
    for offset in (READ_START..READ_END).step_by(source.len()) {
        let len = source.len().min(READ_END - offset);
        fill_expected(offset, &mut source[..len]);
        digest.update(&source[..len]);
    }
    hex(&digest.finalize())
}

fn fill_expected(start: usize, target: &mut [u8]) {
    for (index, byte) in target.iter_mut().enumerate() {
        *byte = expected_byte(start + index);
    }
}

fn expected_byte(index: usize) -> u8 {
    let index = index as u64;
    let mixed = index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    (mixed ^ (mixed >> 17) ^ (index >> 11) ^ ((index >> 18) * 37)) as u8
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex_16(encoded: &str) -> [u8; 16] {
    assert_eq!(encoded.len(), 32);
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
