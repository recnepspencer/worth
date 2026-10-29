//! Ordering the retirements above the checkpoint reads the root manifest and
//! routing blocks of every generation among them. One that outgrew its page
//! is damaged media. Recovery must say so and must not name a limit: an
//! operator would raise a limit on a corrupt store.
//!
//! A limit comes from a budget the caller set. No artifact's own ceiling is
//! one, and neither is a length read off media.

#[allow(unused_imports)] use crate::production_entry::published_above_checkpoint::damaged_history::*;
use {std::path::Path, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::published_above_checkpoint};
use {};
use {std::fs};
use {std::path::PathBuf};
use {worth_store_physical_format::RecordArtifactFile};
use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::WorthStoreRecovery};

#[test]
fn an_oversized_root_or_routing_block_of_the_walked_history_is_damage_not_a_limit() {
    let world =
        pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle);
    let roots = world.root().join("families/records/roots");
    let selected = (1..=4096)
        .rev()
        .find(|generation| root_manifest(&roots, *generation).exists())
        .expect("a published root");
    // The walk reads the roots below the selected one. Source selection
    // itself reads the routing of the root just below, so the routing blocks
    // come from the root under that.
    let mut damaged = vec![
        root_manifest(&roots, selected - 1),
        root_manifest(&roots, selected - 2),
    ];
    let blocks = routing_blocks(&roots, selected - 2);
    assert!(!blocks.is_empty());
    damaged.extend(blocks);
    for artifact in &damaged {
        let outcome = recover_with_oversized(world.root(), artifact);
        let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("{artifact:?} oversized must block: {outcome:?}")
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let evidence = blocked.evidence();
        assert!(
            blocked.cause().damage() == Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::PageAdmission)
                && matches!(
                    evidence.planning_denial,
                    Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Page(
                        worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial::InvalidTarget(_)
                    ))
                ),
            "{artifact:?} oversized is damage: {:?} {:?}",
            blocked.cause(),
            evidence.planning_denial,
        );
        assert_eq!(
            blocked.cause().limit(),
            None,
            "{artifact:?} oversized names no limit"
        );
    }
    // Every artifact is back as the kill left it.
    let serving = crate::production_entry::published_above_checkpoint::serve(&world, "after the damaged attempts");
    world.assert_objects_read_back(&serving);
    serving.close();
}

/// Source selection reads the routing of the selected root and of the root
/// below it, each block under the page its format declares. A block past
/// that page is damaged however many manifest bytes recovery still admits.
#[test]
fn an_oversized_routing_block_of_the_selected_roots_is_damage_not_a_limit() {
    let world =
        pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle);
    let roots = world.root().join("families/records/roots");
    let selected = (1..=4096)
        .rev()
        .find(|generation| root_manifest(&roots, *generation).exists())
        .expect("a published root");
    let mut damaged = routing_blocks(&roots, selected);
    damaged.extend(routing_blocks(&roots, selected - 1));
    assert!(damaged.len() >= 2);
    for artifact in &damaged {
        let outcome = recover_with_oversized(world.root(), artifact);
        let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("{artifact:?} oversized must block: {outcome:?}")
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let evidence = blocked.evidence();
        assert_eq!(
            blocked.cause().damage(),
            Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::SourceSelection),
            "{artifact:?} oversized is damage: {:?}",
            evidence.source_denials,
        );
        assert_eq!(
            blocked.cause().limit(),
            None,
            "{artifact:?} oversized names no limit"
        );
    }
    let serving = crate::production_entry::published_above_checkpoint::serve(&world, "after the damaged attempts");
    world.assert_objects_read_back(&serving);
    serving.close();
}
#[cfg(all())]
#[path = "ceilings/mod.rs"]
pub(crate) mod ceilings;
