pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::EXTENT_PAYLOAD_BYTES, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction, std::path::Path, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_file, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::prepare_extent_rewrite, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::post_reopen::observe_release_fixture, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::post_reopen::reseal_observer_fixture_frame};

pub(crate) const CHILD: &str =
    "process::store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::extent_release_child";
pub(crate) const MARKER: &str = "C11_EXTENT_RELEASE_MARKER";
pub(crate) const SEAM: &str = "C11_EXTENT_RELEASE_SEAM";
pub(crate) const RESUME: &str = "C11_EXTENT_RELEASE_RESUME";

pub(crate) fn finish_reopened_release(root: &Path) {
    let before = produced_retirement_payloads(root);
    assert_eq!(before.len(), 1);
    let intent = &before[0];
    let roots = intent.release_roots.unwrap();
    let range = intent.arena_range.unwrap();
    let arena = std::fs::read(arena_file(root, range[0])).unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(root);
    serving
        .retire_displaced_segment()
        .expect("durable exact release resumes");
    assert_eq!(std::fs::read(arena_file(root, range[0])).unwrap(), arena);
    serving.close();
    let after = produced_retirement_payloads(root);
    assert_eq!(
        after.len(),
        2,
        "exactly the original intent and one completion remain"
    );
    let completion = after.last().unwrap();
    assert_eq!(completion.action, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion);
    assert_eq!(completion.release_roots, Some(roots));
    assert_eq!(completion.release_digest, intent.release_digest);
    assert_eq!(completion.arena_range, intent.arena_range);
    let completed_count = after
        .iter()
        .filter(|record| record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion)
        .count();
    assert_eq!(completed_count, 1);
    use sha2::Digest;
    let root_bytes = std::fs::read(root.join(format!(
        "families/records/roots/root-{:016x}.manifest",
        roots[1]
    )))
    .unwrap();
    assert_eq!(
        Some(<[u8; 32]>::from(sha2::Sha256::digest(root_bytes))),
        intent.release_digest
    );
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(root);
    assert!(matches!(
        serving.retire_displaced_segment(),
        Ok(()) | Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent)
    ));
    assert_eq!(
        produced_retirement_payloads(root).len(),
        after.len(),
        "second reopen must not double-release"
    );
    let (_, placement, _) = configuration();
    let replacement = vec![137; EXTENT_PAYLOAD_BYTES];
    let appended = completed(prepare(&serving, placement, [117; 32], &replacement).execute());
    let identity = appended.persisted_records()[0];
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let reused = current_extent_route(root, identity);
    assert_eq!([reused.arena, reused.offset, reused.length], range);
    assert_eq!(read_record(&serving, record), replacement);
    serving.close();
    crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::post_reopen::assert_clean_arena_accounting(root);
}

pub(crate) fn kill_child(seam: u8) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    kill_at_existing(directory.path(), seam, false);
    let root = directory.path().join("store");
    (directory, root)
}

pub(crate) fn kill_at_existing(directory: &Path, seam: u8, resume: bool) {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    if resume {
        command.env(RESUME, "1");
    }
    let mut child = command
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .env(SEAM, seam.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "release seam {seam} not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}
