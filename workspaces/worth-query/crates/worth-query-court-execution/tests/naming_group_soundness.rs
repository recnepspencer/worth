//! Group naming fails closed and group reads stale exactly (#334): a claim
//! that changes a scheme's continuity rule refuses typed, and a tracked group
//! read is stale after any commit that can change its answer, including an
//! undecodable frame, a revived group on a walked path, and a carrier joining
//! a live successor a bounded selection did not match.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNameReader, ExecutionNameResolveDenial,
    ExecutionNamingCommitAuthority, ExecutionNamingDenial, ExecutionPendingEntityBirth,
    NameReadCurrentness, NameReadDependencies,
};
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupNamingDenial, AggregateGroupScheme,
    AggregateGroupSchemeId, AggregateIdentity, AggregateMemberRole,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    LineageNameDenial, PersistentNameDurabilityError, PersistentNameIdentity,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::CommitResult;

use crate::naming_execution_fixture::{
    begin_branch_staged_tx, create_face_intent, execution_lease_for_workers, face_ref,
    fresh_cad_runtime, lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face,
    RoomGroup, FACE_SCHEME_ID, ROOM_GROUP_SCHEME_ID,
};

type Named = (WorthQueryName<Face>, PersistentNameIdentity);
type Members = Vec<(PersistentNameIdentity, AggregateMemberRole)>;
type Room = WorthQueryName<RoomGroup>;

/// One claim: the held rooms it dissolves and its declared partition.
struct Claim {
    rule: AggregateContinuityRule,
    dissolved: Vec<Room>,
    groups: Vec<Members>,
}

fn rooms(groups: Vec<Members>) -> Claim {
    Claim {
        rule: AggregateContinuityRule::Boundary,
        dissolved: Vec::new(),
        groups,
    }
}

fn wall(face: &Named) -> (PersistentNameIdentity, AggregateMemberRole) {
    (face.1, AggregateMemberRole::Boundary)
}

/// Commits faces `keys` and `claims` on main; returns the commit and names.
fn try_commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    token: &str,
    keys: &[&str],
    claims: Vec<Claim>,
) -> Result<(CommitResult, Vec<Named>), ExecutionNamingDenial> {
    let catalog = lowered_cad_catalog();
    let intents = keys.iter().map(|key| create_face_intent(key)).collect();
    let tx = begin_branch_staged_tx(runtime, &runtime.main_branch_identity(), token, intents);
    let mut plan = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    for key in keys {
        let birth =
            ExecutionPendingEntityBirth::new(face_ref(key), FACE_SCHEME_ID, *key, "authored");
        plan.add_entity_birth(birth).unwrap();
    }
    let plan = claims.into_iter().fold(plan.build(), |plan, claim| {
        let id = AggregateGroupSchemeId::new(ROOM_GROUP_SCHEME_ID).unwrap();
        let dissolved = claim.dissolved.iter();
        let dissolved = dissolved.map(|room| AggregateIdentity::from_bytes(*room.as_bytes()));
        let scheme = AggregateGroupScheme::new(id, claim.rule);
        plan.with_group_claim(scheme, dissolved.collect(), claim.groups)
    });
    let (result, names) =
        ExecutionNamingCommitAuthority::commit_with_naming_plan(runtime, execution, tx, plan)?
            .into_parts();
    let named = keys
        .iter()
        .map(|key| {
            let (id, name) = names
                .created_entity_name::<CadNamingSchema, Face>(&result, &face_ref(key))
                .unwrap();
            (name, names.entity_relational_identity(id).unwrap())
        })
        .collect();
    release_commit_snapshot(runtime, &result);
    Ok((result, named))
}

fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    token: &str,
    keys: &[&str],
    claims: Vec<Claim>,
) -> (CommitResult, Vec<Named>) {
    try_commit(runtime, execution, token, keys, claims)
        .unwrap_or_else(|denial| panic!("{token}: {denial:?}"))
}

/// Runs `read` with a reader at main's head.
fn at_head<T>(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    read: impl FnOnce(&ExecutionNameReader<'_>) -> T,
) -> T {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    read(&ExecutionNameReader::at_observation(
        runtime,
        execution,
        &observation,
    ))
}

fn room_of(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    face: &Named,
) -> Room {
    at_head(runtime, execution, |reader| {
        let mut deps = NameReadDependencies::new();
        reader.group_name_of::<RoomGroup, Face>(&face.0, &mut deps)
    })
    .unwrap()
    .expect("the face is in a room")
}

fn main_branch() -> BranchId {
    BranchId("main".to_owned())
}

/// A room `g1 = {a}` merged into `g2 = {b, c}`: `g2` keeps by its wall
/// majority and `g1` is absorbed. Returns the faces `a, b, c, d` and both.
fn merged(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> (Vec<Named>, Room, Room) {
    let (_, faces) = commit(
        runtime,
        execution,
        "seed",
        &["a", "b", "c", "d"],
        Vec::new(),
    );
    let split = vec![
        vec![wall(&faces[0])],
        vec![wall(&faces[1]), wall(&faces[2])],
    ];
    commit(runtime, execution, "rooms", &[], vec![rooms(split)]);
    let (g1, g2) = (
        room_of(runtime, execution, &faces[0]),
        room_of(runtime, execution, &faces[1]),
    );
    let whole = vec![faces[..3].iter().map(wall).collect()];
    commit(runtime, execution, "merge", &[], vec![rooms(whole)]);
    assert_eq!(
        room_of(runtime, execution, &faces[0]),
        g2,
        "the wall majority keeps"
    );
    (faces, g1, g2)
}

/// A claim under another continuity rule than the scheme's history refuses
/// typed, and commits nothing.
#[test]
fn a_cross_commit_continuity_rule_change_refuses_typed() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let (faces, _, kept) = merged(&runtime, &execution);
    let changed = |groups| Claim {
        rule: AggregateContinuityRule::Anchor,
        dissolved: Vec::new(),
        groups,
    };
    // Unchanged membership and a changed one both refuse.
    for groups in [
        vec![faces[..3].iter().map(wall).collect()],
        vec![vec![wall(&faces[3])]],
    ] {
        let denial = try_commit(&runtime, &execution, "anchor", &[], vec![changed(groups)])
            .map(|_| ())
            .expect_err("a changed continuity rule never names");
        assert!(
            matches!(
                denial.lineage_name_denial(),
                Some(LineageNameDenial::GroupNaming {
                    scheme_id,
                    denial: AggregateGroupNamingDenial::ContinuityRuleChanged,
                }) if scheme_id == ROOM_GROUP_SCHEME_ID
            ),
            "{denial:?}"
        );
    }
    assert_eq!(room_of(&runtime, &execution, &faces[0]), kept);
    let ungrouped = at_head(&runtime, &execution, |reader| {
        reader.group_name_of::<RoomGroup, Face>(&faces[3].0, &mut NameReadDependencies::new())
    });
    assert_eq!(
        ungrouped.unwrap(),
        None,
        "the refused claim grouped nothing"
    );
}

/// An undecodable frame stales a read that found a member ungrouped.
#[test]
fn an_undecodable_frame_stales_an_ungrouped_member_read() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let (faces, _, _) = merged(&runtime, &execution);
    let mut deps = NameReadDependencies::new();
    let answer = at_head(&runtime, &execution, |reader| {
        reader.group_name_of::<RoomGroup, Face>(&faces[3].0, &mut deps)
    });
    assert_eq!(answer.unwrap(), None, "d is in no room");
    let undecodable = Err(PersistentNameDurabilityError::TrailingBytes { remaining: 1 });
    assert!(matches!(
        deps.currentness_after_group_lineage(&main_branch(), undecodable),
        NameReadCurrentness::Stale(_)
    ));
}

/// Reviving a retired group on a walked path stales the walk: `g1` walks to
/// `g2`, which a dissolve retired; re-forming `g2` revives it.
#[test]
fn reviving_a_walked_retired_group_stales_the_resolution() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let (faces, g1, g2) = merged(&runtime, &execution);
    let dissolve = Claim {
        rule: AggregateContinuityRule::Boundary,
        dissolved: vec![g2],
        groups: Vec::new(),
    };
    commit(&runtime, &execution, "dissolve", &[], vec![dissolve]);
    let mut deps = NameReadDependencies::new();
    let retired = at_head(&runtime, &execution, |reader| {
        reader.resolve_group::<RoomGroup>(&g1, &NameSelection::Exact, &mut deps)
    });
    assert!(
        matches!(retired, Ok(ResolvedName::Retired { .. })),
        "g1 walks to the retired g2: {retired:?}"
    );
    let reformed = vec![vec![wall(&faces[1]), wall(&faces[2])]];
    let (revival, _) = commit(&runtime, &execution, "revive", &[], vec![rooms(reformed)]);
    assert_eq!(
        room_of(&runtime, &execution, &faces[1]),
        g2,
        "g2 is revived"
    );
    assert!(matches!(
        deps.currentness_after_commit(&revival),
        NameReadCurrentness::Stale(_)
    ));
}

/// A bounded selection that matched no live successor is stale once its
/// carrier joins that successor.
#[test]
fn a_carrier_joining_an_unmatched_successor_stales_the_resolution() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let (faces, g1, g2) = merged(&runtime, &execution);
    let bounded = NameSelection::bounded_by_carriers([faces[3].0.erased()]).unwrap();
    let mut deps = NameReadDependencies::new();
    let unmatched = at_head(&runtime, &execution, |reader| {
        reader.resolve_group::<RoomGroup>(&g1, &bounded, &mut deps)
    });
    assert!(
        matches!(
            unmatched,
            Err(ExecutionNameResolveDenial::Resolution(
                NameResolutionDenial::NoMatchingCandidate { .. }
            ))
        ),
        "g2 lacks d: {unmatched:?}"
    );
    let joined = vec![faces.iter().map(wall).collect()];
    let (join, _) = commit(&runtime, &execution, "join", &[], vec![rooms(joined)]);
    assert_eq!(room_of(&runtime, &execution, &faces[3]), g2, "d joins g2");
    assert!(matches!(
        deps.currentness_after_commit(&join),
        NameReadCurrentness::Stale(_)
    ));
}
