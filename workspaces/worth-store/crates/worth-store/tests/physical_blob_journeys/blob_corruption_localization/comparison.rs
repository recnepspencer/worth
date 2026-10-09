use std::{path::Path, process::Command, time::Duration};

use worth_store::integrity_observation::{
    PhysicalIntegrityRuntimeReportContext, PhysicalIntegrityRuntimeReportDenial,
};
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubRequest, PhysicalIntegrityScrubTarget, ServingPhysicalRuntime,
};
use worth_store_offline_integrity_observer::{
    compare_selected_integrity_observations, PhysicalIntegrityComparisonLimits,
};
use worth_store_physical_format::{PersistedRecordIdentity, RootPublicationCell};

pub(super) fn observe_selected(root: &Path) -> String {
    let binary = std::env::var_os("WORTH_C9_OBSERVER_EXECUTABLE")
        .map(std::path::PathBuf::from)
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
    let output = Command::new(&binary)
        .arg("observe-selected")
        .arg("--store-root")
        .arg(root)
        .args([
            "--report",
            "-",
            "--max-entries",
            "4096",
            "--max-bytes",
            "268435456",
            "--max-open-files",
            if cfg!(windows) { "6" } else { "5" },
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "600000",
            "--max-report-bytes",
            "2097152",
            "--run",
            "offline-c11-middle-chunk",
            "--scenario",
            "damaged",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "standalone selected observer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let selected: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        selected["protocol"],
        "store.physical.selected-integrity-observation"
    );
    assert_eq!(selected["role"], "offline-root-observer");
    assert_ne!(selected["process"], std::process::id().to_string());
    String::from_utf8(output.stdout).unwrap()
}

pub(super) fn assert_selected_join(
    serving: &ServingPhysicalRuntime,
    target: PhysicalIntegrityScrubTarget,
    selected_root: RootPublicationCell,
    record: PersistedRecordIdentity,
    offline_selected: &str,
) {
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let before_media = serving.media_counters();
    let mut v1_output = Vec::new();
    assert!(matches!(
        scrub.write_observation_report(
            PhysicalIntegrityRuntimeReportContext::new("v1-denial", "damaged").unwrap(),
            16_384,
            &mut v1_output,
        ),
        Err(PhysicalIntegrityRuntimeReportDenial::SelectedTargetRequiresVersionTwo)
    ));
    assert!(v1_output.is_empty());
    assert_eq!(serving.media_counters(), before_media);
    assert_eq!(scrub.counters().completed_windows, 0);
    let mut wire = Vec::new();
    let written = scrub
        .write_selected_record_observation_report(
            PhysicalIntegrityRuntimeReportContext::new("c11-middle-chunk", "damaged").unwrap(),
            16_384,
            &mut wire,
        )
        .unwrap();
    assert_eq!(written, wire.len() as u64);
    let selected: serde_json::Value = serde_json::from_slice(&wire).unwrap();
    assert_eq!(
        selected["protocol"],
        "store.physical.selected-integrity-observation"
    );
    assert_eq!(selected["version"], 2);
    assert_eq!(selected["role"], "runtime-selected-scrub");
    assert_eq!(selected["completeness"], "complete");
    assert_eq!(selected["consumed"]["report_bytes"], written);
    assert_eq!(
        selected["selected_root"]["generation"],
        selected_root.generation().get()
    );
    assert_eq!(
        selected["selected_root"]["reference"],
        selected_root.root_reference().get()
    );
    assert_eq!(selected["artifacts"][0]["family"], "blob_chunk_frame");
    assert_eq!(
        selected["artifacts"][0]["record"],
        super::hex_record(record)
    );
    assert_eq!(selected["artifacts"][0]["outcome"]["posture"], "damaged");
    assert!(selected["artifacts"][0].get("path").is_none());
    assert!(selected["artifacts"][0].get("physical_path").is_none());

    let compared = compare_selected_integrity_observations(
        std::str::from_utf8(&wire).unwrap(),
        offline_selected,
        PhysicalIntegrityComparisonLimits::default(),
    )
    .unwrap();
    let comparison: serde_json::Value = serde_json::from_str(compared.encoded_report()).unwrap();
    let joined = comparison["comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["record"] == super::hex_record(record))
        .expect("independent selected observer must find the same record");
    assert_eq!(joined["runtime_family"], "blob_chunk_frame");
    assert_eq!(joined["offline_family"], "blob_chunk_frame");
    assert!(joined["posture_disagreement"].is_null(), "{joined}");
    assert!(
        !joined["different_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "disposition"),
        "{joined}"
    );
}
