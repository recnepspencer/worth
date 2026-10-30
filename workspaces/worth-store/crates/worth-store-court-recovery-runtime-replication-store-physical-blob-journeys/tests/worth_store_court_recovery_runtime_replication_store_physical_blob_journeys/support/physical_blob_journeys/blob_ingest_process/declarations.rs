pub(crate) use {std::num::NonZeroU64, std::path::Path, std::process::Command};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
#[cfg(not(all()))]
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

pub(crate) use worth_store_testkit_store::allocation_probe::physical_blob_journeys::peak_live_bytes_during;
pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) use crate::physical_blob_journeys::blob_ingest_process::range_read::reader;

pub(crate) const OBJECT_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const SOURCE_WINDOW: usize = 1024 * 1024;
pub(crate) const CHUNK_BYTES: usize = 256 * 1024;
pub(crate) const READ_START: usize = 128 * 1024;
pub(crate) const READ_END: usize = 1_152 * 1024;
pub(crate) const SELECTED_SCAN_LIMIT: u64 = 512;
pub(crate) const SCOPE_KEY: &str = "c11.blob.process.scope";
#[cfg(not(all()))]
pub(crate) const ROLE_ENV: &str = "WORTH_STORE_C11_BLOB_CHILD_ROLE";
#[cfg(not(all()))]
pub(crate) const ROOT_ENV: &str = "WORTH_STORE_C11_BLOB_CHILD_ROOT";
pub(crate) const PUBLICATION_ENV: &str = "WORTH_STORE_C11_BLOB_PUBLICATION";
pub(crate) const PUBLICATION_PREFIX: &str = "C11_BLOB_PUBLICATION ";
pub(crate) const READ_PREFIX: &str = "C11_BLOB_READ ";
pub(crate) const PROCESS_PREFIX: &str = "C11_BLOB_PROCESS ";
pub(crate) const NEGATIVE_PREFIX: &str = "C11_BLOB_NEGATIVE_PEAK ";

pub(crate) fn writer(root: &Path) {
    println!("{PROCESS_PREFIX}{}", std::process::id());
    let serving = serving_from_initialization(root);
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(SELECTED_SCAN_LIMIT).unwrap(),
    );
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        OBJECT_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
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

pub(crate) fn allocator_negative_control() {
    let (_, peak) = peak_live_bytes_during(|| {
        let whole = vec![0_u8; OBJECT_BYTES];
        std::hint::black_box(&whole);
    });
    println!("{NEGATIVE_PREFIX}{peak}");
}

pub(crate) fn observe_closed_store_named(
    root: &Path,
    run: &str,
    scenario: &str,
) -> serde_json::Value {
    observe_closed_store_with_limits(root, run, scenario, 4096, 268_435_456, 2_097_152, 600_000)
}

/// A complete offline observation in which no artifact is damaged. Returns
/// the artifacts. It says nothing about an artifact the observer did not
/// reach: a caller that means "these rows are intact" asserts those rows.
pub(crate) fn observed_without_damage(
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
pub(crate) fn selected_root(artifacts: &[serde_json::Value]) -> Vec<&serde_json::Value> {
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

pub(crate) fn observe_closed_store_with_limits(
    root: &Path,
    run: &str,
    scenario: &str,
    maximum_entries: u64,
    maximum_bytes: u64,
    maximum_report_bytes: u64,
    maximum_elapsed_milliseconds: u64,
) -> serde_json::Value {
    let binary = worth_store_test_support::OFFLINE_ROOT_OBSERVER.path();
    let maximum_open_files = if cfg!(windows) { "6" } else { "5" };
    let output = std::process::Command::new(&binary)
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

pub(crate) fn fill_expected(start: usize, target: &mut [u8]) {
    for (index, byte) in target.iter_mut().enumerate() {
        *byte = expected_byte(start + index);
    }
}

pub(crate) fn expected_byte(index: usize) -> u8 {
    let index = index as u64;
    let mixed = index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    (mixed ^ (mixed >> 17) ^ (index >> 11) ^ ((index >> 18) * 37)) as u8
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn unhex_16(encoded: &str) -> [u8; 16] {
    assert_eq!(encoded.len(), 32);
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
