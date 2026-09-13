use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};

use super::super::harness::compare_runtime_and_observer_with_budget;
use super::super::harness::ProcessWorld;
use super::successor_candidate_media::{candidate_root_path, candidate_topology};

#[test]
fn killed_writer_adopts_exact_multi_level_capacity_transition_candidate() {
    let world = ProcessWorld::start_capacity_transition_crash(0xC8_09_00_47, 0xC8_19_00_47);
    let selected_generation = world
        .writer
        .history
        .current_root_generation()
        .expect("writer leaves a selected root");
    let candidate_generation = selected_generation + 1;
    let records = world.writer.root.join("families/records/roots");
    let selected_bytes = std::fs::read(
        records.join(
            RecordArtifactFile::RootManifest {
                generation: selected_generation,
            }
            .file_name(),
        ),
    )
    .expect("read selected root");
    let candidate_bytes = std::fs::read(candidate_root_path(
        &world.writer.root,
        candidate_generation,
    ))
    .expect("read capacity-transition candidate root");
    let (selected, _) = DurablePhysicalRootManifest::decode(&selected_bytes, u16::MAX)
        .expect("decode selected root");
    let (candidate, _) = DurablePhysicalRootManifest::decode(&candidate_bytes, u16::MAX)
        .expect("decode candidate root");
    assert_eq!(selected.node_capacity(), 64);
    assert_eq!(candidate.node_capacity(), 4);
    assert!(
        candidate
            .routing_root()
            .is_some_and(|root| root.level() >= 2),
        "capacity transition must leave a multi-level root routing tree"
    );
    assert!(
        candidate
            .segment_root()
            .is_some_and(|root| root.generation() == candidate_generation),
        "capacity transition must rewrite the segment tree into the successor generation"
    );
    let candidate_before = candidate_topology(&world.writer.root, candidate_generation);

    let runtime = world.recover_root_with_profile(
        &world.writer.root,
        "capacity-transition-recovery",
        "c8-phase8-fate-coverage-v1",
    );
    let observer = world.observe("capacity-transition-reopened");
    let history = world.parent_history_after_recovery(&world.writer.root);
    compare_runtime_and_observer_with_budget(&runtime, &observer, &history, 4 * 1024 * 1024);
    assert_eq!(
        candidate_before,
        candidate_topology(&world.writer.root, candidate_generation),
        "recovery must preserve the writer's exact capacity-transition bytes"
    );

    let second = world.recover_root_with_profile(
        &world.writer.root,
        "capacity-transition-second-recovery",
        "c8-phase8-fate-coverage-v1",
    );
    let second_observer = world.observe("capacity-transition-second-reopened");
    let second_history = world.parent_history_after_recovery(&world.writer.root);
    compare_runtime_and_observer_with_budget(
        &second,
        &second_observer,
        &second_history,
        4 * 1024 * 1024,
    );
    assert_eq!(
        candidate_before,
        candidate_topology(&world.writer.root, candidate_generation),
        "fresh reopen must converge without rewriting adopted capacity-transition bytes"
    );
}
