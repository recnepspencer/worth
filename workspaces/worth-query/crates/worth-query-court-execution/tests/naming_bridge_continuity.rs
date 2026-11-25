//! Bridge continuity of named records consumes public persistent-name
//! resolution: a UI selection holding a named face reports the topology and
//! the successor records Relational's name index resolves at the retained
//! head, and only live lineages are reported as resolved.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::NameReadDependencies;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::CommitResult;
use worth_runtime_bridge::facade::{
    BridgeHistoricalLineageTopology as Topology, BridgeHistoricalResolvedRecordIdentity,
    RelationalBridgeRecordIdentityParts,
};

use crate::naming_bridge_planning::{lifecycle_face_runtime, RetainedBridge};
use crate::naming_execution_fixture::{execution_lease_for_workers, CadNamingSchema, Face};
use crate::naming_tracked_world::{NamedFace, Rebirth, TrackedWorld};

fn parts(entity: EntityId) -> RelationalBridgeRecordIdentityParts {
    RelationalBridgeRecordIdentityParts::entity(
        entity.partition_id.0,
        entity.local_slot.0,
        entity.generation.0,
    )
}

fn records(entities: &[EntityId]) -> Vec<BridgeHistoricalResolvedRecordIdentity> {
    let mut records: Vec<_> = entities
        .iter()
        .map(|entity| {
            BridgeHistoricalResolvedRecordIdentity::from_relational_record(parts(*entity))
        })
        .collect();
    records.sort_unstable();
    records
}

/// What public name resolution says `held` designates at the head.
fn resolved(
    world: &TrackedWorld,
    execution: &ExecutionRequest<'_, '_>,
    held: &NamedFace,
) -> Vec<EntityId> {
    let head = world.head();
    let resolved = world
        .reader(execution, &head)
        .resolve_entity::<CadNamingSchema, Face>(
            &held.name,
            &NameSelection::AllDescendants,
            &mut NameReadDependencies::new(),
        );
    match resolved.expect("held name resolves") {
        ResolvedName::Exact(entity) => vec![entity],
        ResolvedName::Plural(entities) => entities,
        ResolvedName::Retired { .. } => Vec::new(),
    }
}

/// Where Bridge says the selection that held `prior` in `commit` went: at
/// `commit`'s own snapshot, or at the latest retained head.
#[derive(Clone, Copy)]
enum At {
    Commit,
    Head,
}

/// Checks the selection holding `prior` against `topology`, `expected` and
/// public name resolution at the same basis.
#[allow(clippy::too_many_arguments)]
fn assert_selection(
    bridge: &mut RetainedBridge,
    world: &TrackedWorld,
    execution: ExecutionRequest<'_, '_>,
    commit: &CommitResult,
    prior: &NamedFace,
    at: At,
    topology: Topology,
    expected: &[EntityId],
) {
    let planned = bridge.planned_continuity(commit, execution);
    let (slice, at_commit) = planned
        .iter()
        .find(|(slice, _)| slice.relational_record_identity_parts() == Some(parts(prior.id)))
        .expect("the selected record has a continuity entry");
    let authority = match at {
        At::Commit => at_commit.clone(),
        At::Head => bridge.at_head(slice, execution),
    };
    assert_eq!(
        authority.topology(),
        topology,
        "selection holding {prior:?}"
    );
    assert_eq!(
        authority.canonical_resolved_record_identities(),
        records(expected)
    );
    if matches!(at, At::Head) {
        assert_eq!(
            records(&resolved(world, &execution, prior)),
            records(expected)
        );
    }
    assert_eq!(
        authority.canonical_resolved_lineage_identities().len(),
        expected.len(),
        "only live successor lineages are resolved"
    );
    assert!(
        authority.traversed_event_ids().is_empty(),
        "names, not events"
    );
}

#[test]
fn named_selection_follows_preserve_split_keep_parent_merge_and_retirement() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::with_runtime(lifecycle_face_runtime());
    let bridge = &mut RetainedBridge::new(&world.runtime);
    let (seeded, seed) = world.seed(&execution, "occ.seed", &["f1"]);
    bridge.retain_head();
    let f1 = seeded[0];
    assert_selection(
        bridge,
        &world,
        execution,
        &seed,
        &f1,
        At::Commit,
        Topology::SingleSuccessor,
        &[f1.id],
    );

    let (pieces, cut) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    bridge.retain_head();
    let (a, b) = (pieces[0], pieces[1]);
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &f1,
        At::Commit,
        Topology::SplitSuccessors,
        &[a.id, b.id],
    );

    let merged = world.merge_keeping(&execution, a, b);
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &merged,
        &b,
        At::Commit,
        Topology::SingleSuccessor,
        &[a.id],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &a,
        At::Head,
        Topology::SingleSuccessor,
        &[a.id],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &f1,
        At::Head,
        Topology::SingleSuccessor,
        &[a.id],
    );

    let retired = world.retire(&execution, a);
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &retired,
        &a,
        At::Commit,
        Topology::NoAuthoritativeSuccessor,
        &[],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &f1,
        At::Head,
        Topology::NoAuthoritativeSuccessor,
        &[],
    );
}

#[test]
fn named_selection_reports_one_live_successor_as_single_whatever_the_path() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::with_runtime(lifecycle_face_runtime());
    let bridge = &mut RetainedBridge::new(&world.runtime);

    let (seeded, _) = world.seed(&execution, "occ.one", &["g"]);
    bridge.retain_head();
    let (only, one_child) = world.split(&execution, seeded[0], "occ.cut.one", &[("c", 1)]);
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &one_child,
        &seeded[0],
        At::Commit,
        Topology::SingleSuccessor,
        &[only[0].id],
    );

    let (seeded, _) = world.seed(&execution, "occ.pair", &["h"]);
    bridge.retain_head();
    let (pair, cut) = world.split(&execution, seeded[0], "occ.cut.pair", &[("x", 1), ("y", 2)]);
    bridge.retain_head();
    world.retire(&execution, pair[1]);
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &seeded[0],
        At::Head,
        Topology::SingleSuccessor,
        &[pair[0].id],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &cut,
        &pair[1],
        At::Head,
        Topology::NoAuthoritativeSuccessor,
        &[],
    );

    let (parents, _) = world.seed(&execution, "occ.mint", &["p", "q"]);
    bridge.retain_head();
    let (minted, merge) = world.merge_minting(&execution, parents[0], parents[1], "m");
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &merge,
        &parents[0],
        At::Commit,
        Topology::SingleSuccessor,
        &[minted.id],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &merge,
        &parents[1],
        At::Head,
        Topology::SingleSuccessor,
        &[minted.id],
    );

    let (seeded, _) = world.seed(&execution, "occ.regen", &["r"]);
    bridge.retain_head();
    let (regenerated, regen) = world.rebirth(
        &execution,
        seeded[0],
        ("occ.regen", "r", "r2"),
        Rebirth::Regenerate,
    );
    bridge.retain_head();
    assert_eq!(
        regenerated.name, seeded[0].name,
        "regeneration keeps the name"
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &regen,
        &seeded[0],
        At::Commit,
        Topology::SingleSuccessor,
        &[regenerated.id],
    );

    let (seeded, _) = world.seed(&execution, "occ.restore", &["s"]);
    bridge.retain_head();
    let retired = world.retire(&execution, seeded[0]);
    bridge.retain_head();
    assert_selection(
        bridge,
        &world,
        execution,
        &retired,
        &seeded[0],
        At::Commit,
        Topology::NoAuthoritativeSuccessor,
        &[],
    );
    let (restored, restore) = world.rebirth(
        &execution,
        seeded[0],
        ("occ.restore", "s", "s2"),
        Rebirth::Restore,
    );
    bridge.retain_head();
    assert_eq!(
        restored.name, seeded[0].name,
        "restoration revives the retired name"
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &restore,
        &restored,
        At::Commit,
        Topology::SingleSuccessor,
        &[restored.id],
    );
    assert_selection(
        bridge,
        &world,
        execution,
        &retired,
        &seeded[0],
        At::Head,
        Topology::SingleSuccessor,
        &[restored.id],
    );
}
