//! Named production family journeys share the full campaign's editor and assertions.
//! These focused entry points diagnose one family without replaying unrelated rows.
use super::{
    process_execution::{fresh_identity, run_subject},
    process_protocol::{ProcessReportPayload, ProcessSubjectRequest},
};

fn run(family: &str) {
    let observer = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).expect("independent observer");
    let world = tempfile::tempdir().unwrap();
    let stores = world.path().join("stores");
    let reports = world.path().join("reports");
    std::fs::create_dir(&stores).unwrap();
    std::fs::create_dir(&reports).unwrap();
    let root = stores.join("production-root");
    let executable = std::env::current_exe().unwrap();
    let producer = run_subject(
        &executable,
        &reports,
        "producer",
        ProcessSubjectRequest::producer(
            fresh_identity("producer-scenario", world.path()),
            fresh_identity("producer-run", world.path()),
            root.clone(),
            reports.join("producer.report"),
        ),
    );
    let ProcessReportPayload::Produced(manifest) = producer.report.payload() else {
        panic!("production process must return its actual closed manifest");
    };
    manifest.require_unchanged(&root).unwrap();
    super::artifact_courtroom::run_family(
        std::path::Path::new(&observer),
        &root,
        &stores,
        &reports,
        family,
        manifest,
    );
}

macro_rules! families {
    ($($family:ident),+ $(,)?) => {$(
        #[test]
        #[ignore = "requires Cargo-built independent observer and fresh production processes"]
        fn $family() { run(stringify!($family)); }
    )+};
}

families!(
    bootstrap_catalog,
    current_root_selector,
    previous_root_selector,
    root_manifest,
    root_routing_block,
    segment_membership_block,
    inline_page,
    extent_manifest,
    extent_chunk,
    free_space_header,
    free_space_membership_block,
    wal_frame,
    checkpoint_stream_header,
    checkpoint_dirty_basis,
    checkpoint_binding_compaction,
    checkpoint_binding,
    checkpoint_footer
);
