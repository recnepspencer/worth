//! Layer-3 tracked-dependency court (RULES §4): deciding whether a commit
//! stales a reader's recorded name dependencies probes only what the commit
//! touched. A reader watching 64 or 640 live names (1x and 10x), facing one
//! commit that splits one of them, allocates the same while the binary's
//! global allocator counts every allocation on the thread; no production
//! counter is involved and the currentness check cannot opt out.

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::NameSelection;
use worth_query_execution_naming::facade::{NameReadCurrentness, NameReadDependencies};

use crate::naming_execution_fixture::{execution_lease_for_workers, CadNamingSchema, Face};
use crate::naming_tracked_world::TrackedWorld;

#[global_allocator]
static COUNTED: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

/// Allocation calls and bytes of one currentness check over `watched`
/// recorded live names, against a commit splitting one of them.
fn currentness_cost(watched: usize) -> (usize, usize) {
    let world = TrackedWorld::new();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let keys: Vec<String> = (0..watched).map(|i| format!("watched.{i}")).collect();
    let mut faces = Vec::new();
    for (chunk, keys) in keys.chunks(64).enumerate() {
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        let lease = execution_lease_for_workers(2);
        let seeding = ExecutionRequest::leased(&lease);
        let occurrence = format!("occ.watched.{chunk}");
        faces.extend(world.seed(&seeding, &occurrence, &keys).0);
    }
    let head = world.head();
    let mut dependencies = NameReadDependencies::new();
    let reader = world.reader(&execution, &head);
    for face in &faces {
        let resolved = reader.resolve_entity::<CadNamingSchema, Face>(
            &face.name,
            &NameSelection::Exact,
            &mut dependencies,
        );
        assert!(resolved.is_ok(), "{resolved:?}");
    }
    let (_, split) = world.split(&execution, faces[0], "occ.cut", &[("p1", 1), ("p2", 2)]);
    let region = Region::new(COUNTED);
    let verdict = dependencies.currentness_after_commit(&split);
    let change = region.change();
    assert!(
        matches!(verdict, NameReadCurrentness::Stale(_)),
        "splitting a watched name stales the reads: {verdict:?}"
    );
    (change.allocations, change.bytes_allocated)
}

#[test]
fn currentness_allocations_are_flat_in_watched_dependencies() {
    let (small_calls, small_bytes) = currentness_cost(64);
    let (large_calls, large_bytes) = currentness_cost(640);
    let calls = large_calls as f64 / small_calls.max(1) as f64;
    let bytes = large_bytes as f64 / small_bytes.max(1) as f64;
    println!("watched 64: {small_calls} calls / {small_bytes} B; watched 640: {large_calls} calls / {large_bytes} B");
    println!("watched 640 vs 64: calls x{calls:.3}, bytes x{bytes:.3}");
    assert!(small_calls > 0, "the check allocates its verdict");
    assert!(
        calls <= 1.1 && bytes <= 1.1,
        "currentness grows with unrelated watched names"
    );
}
