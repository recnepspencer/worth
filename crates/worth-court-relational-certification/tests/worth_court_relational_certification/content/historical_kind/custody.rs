//! Original funding follows actual admitted bases, never a copied counter.
use super::{
    checkpoint,
    fixture::{snapshot, World},
    funding::request,
};
use crate::content::cold_envelope::Source;
use crate::leased_preparation::ordinary_context::support;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::*;

#[test]
fn original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen() {
    let seed = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(512 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let seed_execution = ExecutionRequest::serial(&seed);
    let original = request(8_000_000);
    let execution = ExecutionRequest::serial(&original);
    let (checkpoint, source, tail, retained_basis) = {
        let mut world = World::new();
        world.create(&seed_execution, 7, "left");
        world.create(&seed_execution, 7, "right");
        let commit = world
            .commit(&execution, world.relation_intent(0, 1, 7, "edge"))
            .unwrap();
        world.release(&commit);
        drop(commit);
        let (_, prior) = world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap();
        let checkpoint = checkpoint::capture(&world.runtime, &seed_execution);
        let covered = checkpoint.covered_position().unwrap();
        let commit = support::update_entity(
            &seed_execution,
            &world.runtime,
            world.entities[0],
            "successor",
        );
        world.release(&commit);
        drop(commit);
        let (_, basis) = world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap();
        let tail = world
            .runtime
            .durability()
            .canonical_commit_artifacts(&seed_execution)
            .map(Result::unwrap)
            .filter(|a| a.position() > covered)
            .map(|a| {
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(a.as_bytes().to_vec())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(tail.len(), 1);
        let source = Arc::new(Source::capture(
            &world.runtime,
            &seed_execution,
            &checkpoint,
            covered,
            None,
        ));
        drop(prior);
        // Runtime destruction leaves the genuine successor basis as sole owner.
        (checkpoint, source, tail, basis)
    };
    let held = original.memory().unwrap().charged_memory_bytes();
    assert_eq!(held, 8_576);
    let reopened = checkpoint::reopen(&seed_execution, &checkpoint, tail, source);
    assert_eq!(original.memory().unwrap().charged_memory_bytes(), held);
    let read_snapshot = snapshot(&reopened);
    let read = reopened
        .read_truth()
        .read_snapshot(&read_snapshot, &seed_execution)
        .unwrap()
        .unwrap();
    assert_eq!(read.entities().len(), 2);
    assert_eq!(read.relations().len(), 1);
    let relation = &read.relations()[0];
    assert_eq!(
        (
            relation.relation_id.partition_id.0,
            relation.relation_id.slot_index(),
            relation.relation_id.generation.0
        ),
        (7, 0, 1)
    );
    assert_eq!(
        checkpoint::truth(&reopened, &seed_execution),
        checkpoint::Truth {
            entities: vec![
                ((7, 0, 1), 1, "successor".into()),
                ((7, 1, 1), 1, "right".into())
            ],
            relations: vec![((7, 0, 1), 2, (7, 0, 1), (7, 1, 1), "edge".into())],
        }
    );
    assert_eq!(
        (
            relation.source.partition_id.0,
            relation.source.slot_index(),
            relation.source.generation.0
        ),
        (7, 0, 1)
    );
    assert_eq!(
        (
            relation.target.partition_id.0,
            relation.target.slot_index(),
            relation.target.generation.0
        ),
        (7, 1, 1)
    );
    drop(read);
    reopened
        .snapshots()
        .release_snapshot(&read_snapshot)
        .unwrap();
    drop(retained_basis);
    assert_eq!(original.memory().unwrap().charged_memory_bytes(), 0);
}
