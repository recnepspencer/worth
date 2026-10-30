#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::*;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_file, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::prepare_extent_rewrite, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

use {crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::post_reopen::observe_release_fixture, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::crash::post_reopen::reseal_observer_fixture_frame};

#[test]
fn killed_extent_release_repeated_resume_keeps_one_original_intent() {
    let (directory, root) = kill_child(1);
    let original = produced_retirement_payloads(&root);
    for _ in 0..2 {
        std::fs::remove_file(directory.path().join("reached")).unwrap();
        kill_at_existing(directory.path(), 1, true);
        let retained = produced_retirement_payloads(&root);
        assert_eq!(
            retained.len(),
            1,
            "resume must synchronize, not reappend, the intent"
        );
        assert_eq!(retained[0].release_digest, original[0].release_digest);
        assert_eq!(retained[0].release_roots, original[0].release_roots);
    }
    finish_reopened_release(&root);
}

#[test]
fn offline_observer_accounts_wal_held_extent_before_release_and_after_root() {
    for seam in [1, 4] {
        let (directory, root) = kill_child(seam);
        let report = observe_release_fixture(&root, directory.path());
        let arena = report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|artifact| artifact["path"].as_str().unwrap().contains("/arenas/"))
            .collect::<Vec<_>>();
        assert!(!arena.is_empty());
        assert!(
            arena
                .iter()
                .all(|artifact| artifact["outcome"]["posture"] == "intact"),
            "{:#?}",
            report["artifacts"]
        );
        assert_eq!(report["completeness"], "complete", "{report:#?}");
    }
}

#[test]
fn offline_observer_checks_held_extent_older_than_previous_without_retirement_intent() {
    let (directory, root) = kill_child(0);
    assert!(produced_retirement_payloads(&root).is_empty());
    let clean = observe_release_fixture(&root, directory.path());
    assert!(
        clean["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with("arena-accounting:")
                && artifact["path"]
                    .as_str()
                    .unwrap()
                    .ends_with("arena-0000000000000001.data")
                && artifact["outcome"]["posture"] == "intact"),
        "the pre-mutation historical source must account cleanly"
    );
    let previous = std::fs::read(root.join("families/records/root-previous.selector")).unwrap();
    assert!(u64::from_le_bytes(previous[65..73].try_into().unwrap()) > 2);
    let clean = observe_release_fixture(&root, directory.path());
    let arenas = clean["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| artifact["path"].as_str().unwrap().contains("/arenas/"))
        .collect::<Vec<_>>();
    assert!(
        arenas
            .iter()
            .all(|artifact| artifact["outcome"]["posture"] == "intact"),
        "{arenas:#?}"
    );
    assert!(arenas
        .iter()
        .any(|artifact| artifact["range"]["offset"] == 0 && artifact["range"]["length"] == 104));
    let path = arena_file(&root, 1);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[4096 + 128] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let damaged = observe_release_fixture(&root, directory.path());
    assert!(
        damaged["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |artifact| artifact["path"].as_str().unwrap().contains("/arenas/")
                    && artifact["range"]["offset"] == 4096
                    && artifact["outcome"]["posture"] == "damaged"
            ),
        "held source must still be read"
    );
}

#[test]
fn offline_observer_rejects_new_route_over_wal_held_historical_source() {
    let (directory, root) = kill_child(0);
    assert!(produced_retirement_payloads(&root).is_empty());
    let u64_at = |bytes: &[u8], offset: usize| {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    };
    let selector = std::fs::read(root.join("families/records/root-current.selector")).unwrap();
    let generation = u64_at(&selector, 65);
    let manifest_path = root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    ));
    let mut manifest = std::fs::read(&manifest_path).unwrap();
    assert_eq!(manifest[88], 1, "fixture has one addressed routing root");
    let block_generation = u64_at(&manifest, 96);
    let block = u64_at(&manifest, 104);
    let block_path = root.join(format!(
        "families/records/roots/root-{block_generation:016x}-block-{block:016x}.manifest"
    ));
    let mut leaf = std::fs::read(&block_path).unwrap();
    assert_eq!(leaf[68], 1, "fixture root is a leaf");
    let count = u16::from_le_bytes(leaf[66..68].try_into().unwrap()) as usize;
    assert!(count >= 3);
    let entry = 88 + (count - 1) * 88;
    assert_eq!(leaf[entry + 24], 2);
    let routed_arena = u64_at(&leaf, entry + 32);
    let routed_offset = u64_at(&leaf, entry + 56) as usize;
    let length = u64_at(&leaf, entry + 64) as usize;
    let routed = std::fs::read(arena_file(&root, routed_arena)).unwrap();
    let mut replacement = vec![0; length];
    let present = routed.get(routed_offset..).unwrap();
    replacement[..present.len().min(length)].copy_from_slice(&present[..present.len().min(length)]);
    let source_path = arena_file(&root, 1);
    let mut source = std::fs::read(&source_path).unwrap();
    source[..length].copy_from_slice(&replacement);
    std::fs::write(source_path, source).unwrap();
    leaf[entry + 32..entry + 40].copy_from_slice(&1_u64.to_le_bytes());
    leaf[entry + 56..entry + 64].copy_from_slice(&0_u64.to_le_bytes());
    reseal_observer_fixture_frame(&mut leaf);
    std::fs::write(&block_path, &leaf).unwrap();
    let checksum = worth_store_testkit_store::durable_frame_oracle::independent_crc32c(&[&leaf]);
    manifest[116..120].copy_from_slice(&checksum.to_le_bytes());
    reseal_observer_fixture_frame(&mut manifest);
    std::fs::write(&manifest_path, manifest).unwrap();
    let report = observe_release_fixture(&root, directory.path());
    let artifacts = report["artifacts"].as_array().unwrap();
    assert!(artifacts.iter().any(|artifact| artifact["path"]
        .as_str()
        .unwrap()
        .ends_with(manifest_path.file_name().unwrap().to_str().unwrap())
        && artifact["outcome"]["posture"] == "intact"));
    assert!(
        artifacts.iter().any(|artifact| artifact["path"]
            .as_str()
            .unwrap()
            .ends_with(block_path.file_name().unwrap().to_str().unwrap())
            && artifact["outcome"]["posture"] == "intact"),
        "resealed root and routing leaf remain admitted independently of arena collision"
    );
    let arenas = artifacts
        .iter()
        .filter(|artifact| {
            artifact["path"].as_str().unwrap().contains("/arenas/")
                || artifact["identity"]
                    .as_str()
                    .unwrap()
                    .starts_with("retirement:")
                || artifact["identity"]
                    .as_str()
                    .unwrap()
                    .starts_with("copy-intent:")
        })
        .map(|artifact| {
            (
                artifact["identity"].as_str().unwrap().to_owned(),
                artifact["range"]["offset"].as_u64(),
                artifact["outcome"].clone(),
            )
        })
        .collect::<Vec<_>>();
    assert!(artifacts.iter().any(|artifact|artifact["identity"].as_str().unwrap().starts_with("arena-accounting:")
        && artifact["path"].as_str().unwrap().ends_with("arena-0000000000000001.data")
        && artifact["outcome"]["posture"] == "damaged"),
        "a resealed current route over protected historical bytes must damage arena accounting: {arenas:#?}");
}

#[test]
fn killed_extent_release_resumes_exact_descriptor_before_candidate_and_after_root() {
    for seam in [1, 4] {
        let (directory, root) = kill_child(seam);
        finish_reopened_release(&root);
        drop(directory);
    }
}

#[test]
fn killed_extent_release_converges_a_matching_partial_candidate() {
    let (directory, root) = kill_child(3);
    let intent = produced_retirement_payloads(&root).pop().unwrap();
    let generation = intent.release_roots.unwrap()[1];
    let candidate = root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    ));
    // The process was killed after actual candidate writes. Model an incomplete
    // last write by retaining an exact prefix, never manufacturing candidate bytes.
    std::fs::OpenOptions::new()
        .write(true)
        .open(candidate)
        .unwrap()
        .set_len(173)
        .unwrap();
    finish_reopened_release(&root);
    drop(directory);
}

#[test]
fn killed_extent_release_rejects_conflicting_candidate_without_publishing() {
    let (directory, root) = kill_child(3);
    let intent = produced_retirement_payloads(&root).pop().unwrap();
    let generation = intent.release_roots.unwrap()[1];
    let candidate = root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    ));
    let mut bytes = std::fs::read(&candidate).unwrap();
    bytes[100] ^= 1;
    std::fs::write(&candidate, &bytes).unwrap();
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert!(serving.retire_displaced_segment().is_err());
    assert_eq!(
        std::fs::read(candidate).unwrap(),
        bytes,
        "conflicting candidate must not be overwritten"
    );
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        catalog
    );
    assert!(produced_retirement_payloads(&root)
        .iter()
        .all(|record| record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent));
    assert!(serving.close_plan().execute().requires_inspection());
    drop(directory);
}

#[test]
#[ignore = "spawned as a producer and killed at a real release boundary"]
fn extent_release_child() {
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
    let seam: u8 = std::env::var(SEAM).unwrap().parse().unwrap();
    let root = marker.join("store");
    let serving = if std::env::var_os(RESUME).is_some() {
        worth_store_testkit_store::physical_record_journeys::serving_from_open(&root)
    } else {
        let serving = serving_from_initialization(&root);
        let (_, placement, _) = configuration();
        let append =
            completed(prepare(&serving, placement, [115; 32], &extent_payload()).execute());
        let record = append.into_acknowledgment().record_ids().next().unwrap();
        completed(prepare_extent_rewrite(&serving, placement, [116; 32], record).execute());
        if seam == 0 {
            completed(prepare(&serving, placement, [118; 32], &extent_payload()).execute());
            completed(prepare(&serving, placement, [119; 32], &extent_payload()).execute());
            std::fs::write(marker.join("reached"), b"1").unwrap();
            loop {
                std::thread::park();
            }
        }
        serving
    };
    let arrived = serving.certification_arm_retirement_kill(seam);
    std::thread::spawn(move || {
        while !arrived.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::fs::write(marker.join("reached"), b"1").unwrap();
    });
    let result = serving.retire_displaced_segment();
    panic!("armed process seam returned: {result:?}");
}
