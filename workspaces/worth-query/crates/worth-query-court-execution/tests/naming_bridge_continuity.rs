//! Bridge continuity of named records consumes public persistent-name
//! resolution: a UI selection holding a named face reports the topology and
//! the successor records Relational's name index resolves, through preserve,
//! split, merge and retirement.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::NameReadDependencies;
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageTopology,
    BridgeHistoricalResolvedRecordIdentity, RelationalBridgeRecordIdentityParts,
};

use crate::naming_bridge_planning::planned_continuity;
use crate::naming_execution_fixture::{execution_lease_for_workers, CadNamingSchema, Face};
use crate::naming_tracked_world::{NamedFace, TrackedWorld};

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

/// The authority Bridge planned for the selection holding `prior`.
fn selection(
    planned: &[(
        Option<RelationalBridgeRecordIdentityParts>,
        BridgeHistoricalLineageAuthority,
    )],
    prior: EntityId,
) -> &BridgeHistoricalLineageAuthority {
    planned
        .iter()
        .find(|(record, _)| *record == Some(parts(prior)))
        .map(|(_, authority)| authority)
        .expect("the selected record has a continuity entry")
}

/// What public name resolution says `held` designates now.
fn resolved_records(
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

#[test]
fn named_selection_continuity_follows_name_resolution_through_split_merge_and_retirement() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::with_runtime(crate::naming_bridge_planning::lifecycle_face_runtime());

    let (seeded, seed) = world.seed(&execution, "occ.seed", &["f1"]);
    let f1 = seeded[0];
    let planned = planned_continuity(&world.runtime, &seed, execution);
    let preserved = selection(&planned, f1.id);
    assert_eq!(
        preserved.topology(),
        BridgeHistoricalLineageTopology::SingleSuccessor
    );
    assert_eq!(
        preserved.canonical_resolved_record_identities(),
        records(&[f1.id])
    );

    let (pieces, cut) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    let planned = planned_continuity(&world.runtime, &cut, execution);
    let split = selection(&planned, f1.id);
    assert_eq!(
        split.topology(),
        BridgeHistoricalLineageTopology::SplitSuccessors
    );
    assert_eq!(
        split.canonical_resolved_record_identities(),
        records(&resolved_records(&world, &execution, &f1)),
        "split candidates agree exactly with public name resolution"
    );
    assert_eq!(
        split.canonical_resolved_record_identities(),
        records(&[a.id, b.id])
    );
    assert!(
        split.traversed_event_ids().is_empty(),
        "name resolution walks names, not events"
    );

    let merged = world.merge_keeping(&execution, a, b);
    let planned = planned_continuity(&world.runtime, &merged, execution);
    let survivor = selection(&planned, b.id);
    assert_eq!(
        survivor.topology(),
        BridgeHistoricalLineageTopology::MergeLikeSuccessor
    );
    assert_eq!(
        survivor.canonical_resolved_record_identities(),
        records(&[a.id])
    );
    assert_eq!(resolved_records(&world, &execution, &b), vec![a.id]);

    let retired = world.retire(&execution, a);
    let planned = planned_continuity(&world.runtime, &retired, execution);
    let gone = selection(&planned, a.id);
    assert_eq!(
        gone.topology(),
        BridgeHistoricalLineageTopology::NoAuthoritativeSuccessor
    );
    assert!(gone.canonical_resolved_record_identities().is_empty());
    assert!(resolved_records(&world, &execution, &f1).is_empty());
}
