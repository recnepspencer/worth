//! Tracked resolution reads only the held name's lineage path: resolving a
//! name three splits deep does the same index work whether the branch holds
//! 1,000 or 20,000 unrelated named records and 50 or 500 unrelated commits.
//!
//! Boundary: the runtime-maintained derived name index read through the
//! public observation view, in a debug build on one machine, warm in memory,
//! one resolution per world. The measured units are the resolver's own
//! index probes, ending transitions read and forward edges followed; they
//! are compared for equality across the two worlds, never pinned to counts.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::NameReadDependencies;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolutionMetrics, PersistentNameSelection,
};

use crate::naming_execution_fixture::{execution_lease_for_workers, CadNamingSchema, Face};
use crate::naming_tracked_world::TrackedWorld;

/// Unrelated named faces committed together; each chunk is one more
/// unrelated commit.
const NOISE_PER_COMMIT: usize = 250;

/// Resolves a name three splits deep amid `records` unrelated named faces
/// and `commits` unrelated commits; returns the resolver's work and how many
/// live descendants the execution reader resolved.
fn resolve_deep_name(records: usize, commits: usize) -> (PersistentNameResolutionMetrics, usize) {
    let world = TrackedWorld::new();
    // Each commit runs under its own request, so no request budget is shared
    // across the world's construction.
    let commit = |seed: &dyn Fn(&ExecutionRequest<'_, '_>)| {
        let lease = execution_lease_for_workers(2);
        seed(&ExecutionRequest::leased(&lease));
    };
    let keys: Vec<String> = (0..records).map(|i| format!("noise.{i}")).collect();
    for chunk in keys.chunks(NOISE_PER_COMMIT) {
        let chunk: Vec<&str> = chunk.iter().map(String::as_str).collect();
        commit(&|execution| drop(world.seed(execution, "occ.noise", &chunk)));
    }
    let held = std::cell::Cell::new(None);
    commit(&|execution| {
        let (seeded, _) = world.seed(execution, "occ.held", &["held"]);
        let (first, _) = world.split(execution, seeded[0], "occ.cut.1", &[("p1", 1), ("p2", 2)]);
        let (second, _) = world.split(execution, first[0], "occ.cut.2", &[("q1", 1), ("q2", 2)]);
        world.split(execution, second[0], "occ.cut.3", &[("r1", 1), ("r2", 2)]);
        held.set(Some(seeded[0]));
    });
    let held = [held.get().expect("held name seeded")];
    for later in 0..commits {
        commit(&|execution| drop(world.seed(execution, &format!("occ.later.{later}"), &["later"])));
    }
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let head = world.head();
    let metrics = world
        .runtime
        .with_persistent_names_at_observation(&execution, &head, |view| {
            let key = view
                .recorded_name(
                    worth_relational::facade::lineage::PersistentNameSubjectKind::Entity,
                    held[0].name.as_bytes(),
                )
                .expect("held name recorded");
            assert!(matches!(key, PersistentNameKey::Entity(_)));
            view.resolve(key, &PersistentNameSelection::AllDescendants)
                .metrics
        })
        .expect("name read admitted");
    let resolved = world
        .reader(&execution, &head)
        .resolve_entity::<CadNamingSchema, Face>(
            &held[0].name,
            &NameSelection::AllDescendants,
            &mut NameReadDependencies::new(),
        );
    let Ok(ResolvedName::Plural(live)) = resolved else {
        panic!("a three-deep split resolves to its live leaves: {resolved:?}");
    };
    (metrics, live.len())
}

#[test]
fn resolution_work_is_flat_in_unrelated_records_and_commits() {
    let (small, small_live) = resolve_deep_name(1_000, 50);
    let (large, large_live) = resolve_deep_name(20_000, 500);
    assert_eq!(small_live, 4, "p2, q2, r1 and r2 are the live leaves");
    assert_eq!(large_live, small_live);
    assert_eq!(
        large, small,
        "twenty times the records and ten times the commits cost the same path work"
    );
}
