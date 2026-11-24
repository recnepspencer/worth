//! Runtime group names survive durable recovery (#334): checkpoint restore
//! and checkpoint + journal tail replay rebuild each branch's exact group
//! state from the canonical group lineage frames in committed envelopes, so a
//! reopened runtime answers every group read with byte-identical names and
//! names a later transition exactly as the source runtime does.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{ApplicationGroupKind, WorthQueryName};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNameReader, ExecutionNamingCommitAuthority,
    ExecutionPendingEntityBirth, NameReadDependencies,
};
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::durability::RecoveryVerificationMode;
use worth_relational::facade::lineage::PersistentNameIdentity;
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

use crate::naming_execution_fixture::{
    begin_branch_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face, RoomGroup, FACE_SCHEME_ID,
    ROOM_GROUP_SCHEME_ID,
};

impl ApplicationGroupKind for RoomGroup {
    const NAME: &'static str = "RoomGroup";
}

type Named = (WorthQueryName<Face>, PersistentNameIdentity);

type Partition = Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>>;

fn room_scheme() -> AggregateGroupScheme {
    let id = AggregateGroupSchemeId::new(ROOM_GROUP_SCHEME_ID).unwrap();
    AggregateGroupScheme::new(id, AggregateContinuityRule::Boundary)
}

fn room(groups: Partition) -> Partition {
    groups
}

/// Commits faces `keys` (and `claims`) on `runtime`; returns their names.
fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    token: &str,
    keys: &[&str],
    claims: Vec<Partition>,
) -> Vec<Named> {
    commit_on(runtime, &runtime.main_branch_identity(), execution, token, keys, claims)
}

fn commit_on(
    runtime: &RelationalRuntime,
    branch: &RelationalBranchIdentity,
    execution: &ExecutionRequest<'_, '_>,
    token: &str,
    keys: &[&str],
    claims: Vec<Partition>,
) -> Vec<Named> {
    let catalog = lowered_cad_catalog();
    let intents = keys.iter().map(|key| create_face_intent(key)).collect();
    let tx = begin_branch_staged_tx(runtime, branch, token, intents);
    let mut plan = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    for key in keys {
        let birth =
            ExecutionPendingEntityBirth::new(face_ref(key), FACE_SCHEME_ID, *key, "authored");
        plan.add_entity_birth(birth).unwrap();
    }
    let plan = claims.into_iter().fold(plan.build(), |plan, groups| {
        plan.with_group_partition(room_scheme(), groups)
    });
    let (result, names) =
        ExecutionNamingCommitAuthority::commit_with_naming_plan(runtime, execution, tx, plan)
            .unwrap_or_else(|denial| panic!("{token}: {denial:?}"))
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
    named
}

/// The room each of `faces` belongs to at `runtime`'s head.
fn rooms(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[Named],
) -> Vec<Option<[u8; 32]>> {
    rooms_on(runtime, &runtime.main_branch_identity(), execution, faces)
}

fn rooms_on(
    runtime: &RelationalRuntime,
    branch: &RelationalBranchIdentity,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[Named],
) -> Vec<Option<[u8; 32]>> {
    let basis = runtime
        .admit_branch_basis(branch)
        .unwrap();
    let observation = basis.observation();
    let reader = ExecutionNameReader::at_observation(runtime, execution, &observation);
    faces
        .iter()
        .map(|(face, _)| {
            let room = reader
                .group_name_of::<RoomGroup, Face>(face, &mut NameReadDependencies::new())
                .unwrap();
            room.map(|room| *room.as_bytes())
        })
        .collect()
}

fn wall(face: &Named) -> (PersistentNameIdentity, AggregateMemberRole) {
    (face.1, AggregateMemberRole::Boundary)
}

fn void(face: &Named) -> (PersistentNameIdentity, AggregateMemberRole) {
    (face.1, AggregateMemberRole::Ordinary)
}

/// Seeds faces, forms one room of three walls and a void, and checkpoints
/// after `checkpoint_after` commits of the two.
fn seeded(
    source: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    checkpoint_after: usize,
) -> Vec<Named> {
    let mut faces = commit(
        source,
        execution,
        "seed",
        &["w1", "w2", "w3", "v"],
        Vec::new(),
    );
    if checkpoint_after == 1 {
        source
            .durability_authority()
            .checkpoint(execution)
            .expect("checkpoint");
    }
    let claim = room(vec![vec![
        wall(&faces[0]),
        wall(&faces[1]),
        wall(&faces[2]),
        void(&faces[3]),
    ]]);
    faces.extend(commit(source, execution, "room", &["spare"], vec![claim]));
    if checkpoint_after == 2 {
        source
            .durability_authority()
            .checkpoint(execution)
            .expect("checkpoint");
    }
    faces
}

/// The same split on two runtimes: `w3` and the void leave the room.
fn split(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[Named],
    token: &str,
) {
    commit(
        runtime,
        execution,
        token,
        &[],
        vec![room(vec![vec![wall(&faces[2]), void(&faces[3])]])],
    );
}

fn recovered(
    source: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalRuntime {
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("recovery plan builds");
    let mut reopened = fresh_cad_runtime();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .expect("recovery succeeds");
    reopened
}

fn assert_recovers(checkpoint_after: usize, split_before_checkpoint: bool) {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let source = fresh_cad_runtime();
    let faces = seeded(&source, &execution, checkpoint_after);
    if split_before_checkpoint {
        split(&source, &execution, &faces, "split");
    }
    let before = rooms(&source, &execution, &faces);
    assert!(before[0].is_some() && before[3].is_some(), "{before:?}");
    let reopened = recovered(&source, &execution);
    assert_eq!(
        rooms(&reopened, &execution, &faces),
        before,
        "recovered group names"
    );
    // A later transition, named by the authority over each runtime's own
    // recovered group state, names the same groups byte for byte.
    if !split_before_checkpoint {
        split(&source, &execution, &faces, "split");
        split(&reopened, &execution, &faces, "split");
    } else {
        let regroup = |runtime: &RelationalRuntime| {
            let claim = room(vec![vec![
                wall(&faces[0]),
                wall(&faces[1]),
                wall(&faces[2]),
                void(&faces[3]),
            ]]);
            commit(runtime, &execution, "merge", &[], vec![claim]);
        };
        regroup(&source);
        regroup(&reopened);
    }
    let after = rooms(&source, &execution, &faces);
    assert_ne!(after, before, "the later transition changed the groups");
    assert_eq!(rooms(&reopened, &execution, &faces), after);
}

#[test]
fn checkpoint_restore_reconstructs_byte_identical_group_names() {
    assert_recovers(2, false);
}

#[test]
fn journal_tail_replay_reconstructs_byte_identical_group_names() {
    assert_recovers(1, true);
}

/// A fork inherits its source's exact groups at the fork commit, and each
/// branch's later transitions stay its own.
#[test]
fn a_fork_inherits_group_names_and_regroups_independently() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let faces = seeded(&runtime, &execution, 0);
    let before = rooms(&runtime, &execution, &faces);
    let (_, source) = runtime.observe_fork_source(&BranchId("main".to_owned())).unwrap();
    let forked = runtime.fork_branch(&execution, BranchId("fork-g".to_owned()), source).unwrap();
    let fork = forked.target_identity().clone();
    assert_eq!(rooms_on(&runtime, &fork, &execution, &faces), before);
    // The fork splits its inherited room; main keeps the whole room.
    let piece = room(vec![vec![wall(&faces[2]), void(&faces[3])]]);
    commit_on(&runtime, &fork, &execution, "fork-split", &[], vec![piece]);
    let on_fork = rooms_on(&runtime, &fork, &execution, &faces);
    assert_eq!(on_fork[0], before[0], "the walls left behind keep the room");
    assert_ne!(on_fork[3], before[3], "the void left with the piece");
    assert_eq!(rooms(&runtime, &execution, &faces), before);
    // Main's own identical split names the same piece the fork named.
    split(&runtime, &execution, &faces, "main-split");
    assert_eq!(rooms(&runtime, &execution, &faces), on_fork);
}
