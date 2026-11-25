//! #119 netting: a successor round supersedes its chain's earlier rounds.
//! Every acceptance here also rebuilds the world from its canonical commits
//! (WAL replay), whose recovery requires each replayed envelope to equal the
//! durable one, and compares the replayed records with the live ones.
use super::*;

mod replay;

const LINK: KindId = KindId(2);

/// A fieldless relation of kind `LINK` from `source` to `target`.
fn link(key: &str, source: EntityReference, target: EntityReference) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: LINK,
        client_key: ClientKey::raw(key),
        source,
        target,
        fields: AspectFieldPatch::default(),
    }))
}

/// Publish `proposal`; a cold WAL replay of the world's whole history must
/// succeed and rebuild every entity the commit changed exactly as live.
fn publish_and_replay(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
) -> Vec<String> {
    let result = world.publish(execution, proposal);
    let ids = changed_entities(&result);
    release_test_commit_snapshot(&world.runtime, &result);
    let live = states(&cold::facts(&world.runtime, execution, &ids));
    let replayed = world
        .replay(execution)
        .unwrap_or_else(|stop| panic!("WAL replay with history parity: {stop:?}"));
    assert_eq!(
        states(&cold::facts(&replayed, execution, &ids)),
        live,
        "every replayed id carries its live record"
    );
    live
}
