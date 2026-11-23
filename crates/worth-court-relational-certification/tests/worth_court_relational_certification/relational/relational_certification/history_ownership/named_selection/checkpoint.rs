//! A restore at the original identity binds a fresh lineage to an entity no
//! commit created. Capture, validation, expanded restore and cold-directory
//! restore must all bind it exactly, beside an unnamed rematerialized entity,
//! and serve the same names afterward.
use super::*;

const PINNED: &str = "pinned-at-delete";

/// The lineage `record_history_at_observation` starts from for `entity`.
fn history_start(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    entity: EntityId,
) -> Option<worth_relational::facade::identity::LineageId> {
    runtime
        .record_history_at_observation(execution, &basis.observation(), entity)
        .unwrap()
        .map(|history| history.start)
}

/// The sibling branch forked at the deletion, as `runtime` serves it.
fn pinned_branch(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    let identity = runtime
        .branch_identity(&BranchId(PINNED.into()))
        .expect("the pinned sibling survives every restore form");
    runtime.observe_branch(&identity).unwrap().1
}

struct Restored {
    runtime: RelationalRuntime,
    named: EntityId,
    plain: EntityId,
    born: AdmittedRelationalBranchBasis,
    name: PersistentNameKey,
    lineages: (
        worth_relational::facade::identity::LineageId,
        worth_relational::facade::identity::LineageId,
    ),
}

fn restored(execution: &ExecutionRequest<'_, '_>) -> Restored {
    let runtime = named_runtime();
    let model = create_entity(execution, &runtime, "model");
    let named = owned_entity(execution, &runtime, &head(&runtime), "named", true);
    let plain = owned_entity(execution, &runtime, &head(&runtime), "plain", false);
    create_relation(execution, &runtime, model, named, "owns-named");
    create_relation(execution, &runtime, model, plain, "owns-plain");
    let born = head(&runtime);
    let PersistentNameOfRecord::Live { name, lineage: l1 } =
        name_of(execution, &runtime, &born, named)
    else {
        panic!("the seeded member is born named");
    };
    assert_eq!(
        name_of(execution, &runtime, &born, plain),
        PersistentNameOfRecord::Unnamed
    );
    delete(execution, &runtime, &[named, plain]);
    // Pin the deletion twice: a held observation and a sibling branch.
    let deleted = head(&runtime);
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(PINNED.into()), source)
        .unwrap();
    let pinned_before = history_start(execution, &runtime, &deleted, named);
    assert_eq!(
        pinned_before,
        Some(l1),
        "the deleted member's history starts at its birth"
    );
    select(execution, &runtime, model, &born).expect("the deletion is undone");
    assert_eq!(
        history_start(execution, &runtime, &deleted, named),
        Some(l1),
        "a later restore never changes the pinned answer"
    );
    let after = head(&runtime);
    let PersistentNameOfRecord::Live {
        name: restored,
        lineage: l2,
    } = name_of(execution, &runtime, &after, named)
    else {
        panic!("the restored member carries its name");
    };
    assert_eq!(restored, name);
    assert_ne!(l1, l2, "a restore binds a fresh lineage");
    assert_eq!(
        name_of(execution, &runtime, &born, named),
        PersistentNameOfRecord::Live { name, lineage: l1 },
        "the pinned birth basis keeps its own lineage"
    );
    Restored {
        runtime,
        named,
        plain,
        born,
        name,
        lineages: (l1, l2),
    }
}

/// What a runtime serves at its head: the restored member's name, the
/// unnamed part, the held name's resolution, and both of the member's nodes.
fn served(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    world: &Restored,
) -> (
    PersistentNameOfRecord,
    PersistentNameOfRecord,
    String,
    [EntityId; 2],
    Option<worth_relational::facade::identity::LineageId>,
) {
    let after = head(runtime);
    let node = |lineage| {
        runtime
            .immutable_lineage_node(execution, lineage)
            .unwrap()
            .expect("every restore epoch keeps its node")
            .node()
            .entity_id()
    };
    (
        name_of(execution, runtime, &after, world.named),
        name_of(execution, runtime, &after, world.plain),
        resolve(execution, runtime, &after, world.name),
        [node(world.lineages.0), node(world.lineages.1)],
        history_start(execution, runtime, &pinned_branch(runtime), world.named),
    )
}

#[test]
fn a_selection_restore_survives_capture_validation_and_both_restore_forms() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = restored(execution);
    let original = served(execution, &world.runtime, &world);
    assert_eq!(original.3, [world.named, world.named]);
    assert_eq!(
        original.4,
        Some(world.lineages.0),
        "the sibling forked at the deletion resolves L1"
    );
    assert!(original.2.starts_with("Ok(Exact("), "{}", original.2);
    let _ = &world.born;

    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .expect("capture binds the restore's entity allocation");
    let mut expanded = named_runtime();
    expanded
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .expect("validation accepts the restore's entity binding");
    assert_eq!(served(execution, &expanded, &world), original);

    let (cold, _) =
        super::super::fixture::cold::reopen(execution, &world.runtime, named_registry());
    assert_eq!(served(execution, &cold, &world), original);
}
