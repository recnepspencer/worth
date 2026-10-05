//! Ordering the retirements above the checkpoint reads the root manifest and
//! routing blocks of every generation among them. One that outgrew its page
//! is damaged media. Recovery must say so and must not name a limit: an
//! operator would raise a limit on a corrupt store.
//!
//! A limit comes from a budget the caller set. No artifact's own ceiling is
//! one, and neither is a length read off media.

use super::super::*;
use pending_wal_world::{Tail, Workload};
use std::fs;
use std::path::PathBuf;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
    PhysicalRecoveryPageAdmissionDenial, PhysicalRecoveryPlanningDenial, WorthStoreRecovery,
};

#[path = "damaged_history/ceilings.rs"]
mod ceilings;

/// More than one page: no root manifest or routing block is this large.
const PAST_ONE_PAGE: usize = (1 << 16) + 1;

fn root_manifest(roots: &Path, generation: u64) -> PathBuf {
    roots.join(RecordArtifactFile::RootManifest { generation }.file_name())
}

/// The routing blocks written with the root of `generation`.
fn routing_blocks(roots: &Path, generation: u64) -> Vec<PathBuf> {
    (1..=generation)
        .map(|block| {
            roots.join(RecordArtifactFile::RootRoutingBlock { generation, block }.file_name())
        })
        .filter(|path| path.exists())
        .collect()
}

/// Recovery over the store while `artifact` is `PAST_ONE_PAGE` bytes longer.
fn recover_with_oversized(root: &Path, artifact: &Path) -> PhysicalRecoveryOutcome {
    recover_with_longer(artifact, PAST_ONE_PAGE, || {
        certified_release_serving::request_with_manifest_entries(root, 4096)
    })
}

/// What `request` recovers while `artifact` is `extra` zero bytes longer.
fn recover_with_longer(
    artifact: &Path,
    extra: usize,
    request: impl FnOnce() -> PhysicalRecoveryOpenRequest,
) -> PhysicalRecoveryOutcome {
    let original = fs::read(artifact).unwrap();
    let mut longer = original.clone();
    longer.resize(original.len() + extra, 0);
    fs::write(artifact, &longer).unwrap();
    let outcome = WorthStoreRecovery::recover(request());
    fs::write(artifact, &original).unwrap();
    outcome
}

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
        let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("{artifact:?} oversized must block: {outcome:?}")
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let evidence = blocked.evidence();
        assert!(
            blocked.kind == PhysicalRecoveryBlockKind::PageAdmission
                && matches!(
                    evidence.planning_denial,
                    Some(PhysicalRecoveryPlanningDenial::Page(
                        PhysicalRecoveryPageAdmissionDenial::InvalidTarget(_)
                    ))
                ),
            "{artifact:?} oversized is damage: {:?} {:?}",
            blocked.kind,
            evidence.planning_denial,
        );
        assert_eq!(
            evidence.limit, None,
            "{artifact:?} oversized names no limit"
        );
    }
    // Every artifact is back as the kill left it.
    let serving = super::serve(&world, "after the damaged attempts");
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
        let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("{artifact:?} oversized must block: {outcome:?}")
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let evidence = blocked.evidence();
        assert_eq!(
            blocked.kind,
            PhysicalRecoveryBlockKind::SourceSelection,
            "{artifact:?} oversized is damage: {:?}",
            evidence.source_denials,
        );
        assert_eq!(
            evidence.limit, None,
            "{artifact:?} oversized names no limit"
        );
    }
    let serving = super::serve(&world, "after the damaged attempts");
    world.assert_objects_read_back(&serving);
    serving.close();
}
