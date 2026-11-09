//! Mixed batches compare SDK law verdicts with a literal candidate graph.
//! Adapted from the independent slice-4 certifier's saved adversarial probe.
mod edits;
mod model;
use super::super::super::cold_checkpoint::{reopen, ColdCheckpointSource};
use model::{Model, Rng};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::RelationalCheckpointArtifactPolicy, transactions::*};
use worth_testkit_relational::maintenance::relation_laws::{request, World};
fn connectivity(error: &TransactionCommitError) -> bool {
    matches!(error, TransactionCommitError::Publication { error, .. }
        if error.detail.contains("source_reaches_target"))
}

#[derive(Default, Debug)]
struct Tally {
    accepted: usize,
    refused_law: usize,
    other: usize,
    entity_deletes: usize,
    replaces: usize,
    creates: usize,
    same_commit_restores: usize,
    unrelated: usize,
    reopens: usize,
}

fn journey(seed: u64, split: bool, steps: usize, tally: &mut Tally) {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = if split {
        World::across_partitions(&execution, 4)
    } else {
        World::new(&execution, 4)
    };
    let mut model = Model::from_world(&world);
    assert!(model.valid());
    let mut rng = Rng(seed);
    for step in 0..steps {
        if step == 30 {
            let checkpoint = world
                .runtime
                .durability_authority()
                .native_checkpoint_with_policy(
                    &execution,
                    execution.allocation_policy(),
                    RelationalCheckpointArtifactPolicy::ExternalMembers {
                        current_members: &[],
                        source: None,
                    },
                )
                .unwrap();
            let source = ColdCheckpointSource::capture(&world.runtime, &execution, &checkpoint);
            world.runtime = reopen(&execution, &checkpoint, source, World::empty_runtime(0));
            tally.reopens += 1;
        }
        let (mut after, batch, log) = edits::build(&model, seed, split, step, &mut rng, tally);
        if log.is_empty() {
            continue;
        }
        let expected = after.valid();
        let branch = worth_relational::facade::history::BranchId("main".into());
        let prior = world.runtime.branch_reference_state(&branch).unwrap();
        let result = world.commit(&execution, batch);
        if result.is_err() {
            assert_eq!(
                world.runtime.branch_reference_state(&branch).unwrap(),
                prior,
                "a refused mixed batch must leave the published head unchanged"
            );
        }
        match result {
            Ok(result) => {
                assert!(
                    expected,
                    "FALSE ACCEPT seed {seed} split {split} step {step}: {log:?}"
                );
                tally.accepted += 1;
                after.bind_created(&result);
                world.release(&result);
                model = after;
            }
            Err(error) if connectivity(&error) => {
                assert!(
                    !expected,
                    "FALSE REFUSAL seed {seed} split {split} step {step}: {log:?}: {error:?}"
                );
                tally.refused_law += 1;
            }
            Err(error) => {
                // These two proposal conflicts arise when distinct intents in a
                // batch collide. They are not evidence of a law verdict.
                assert!(
                    matches!(&error, TransactionCommitError::Conflict { error, .. }
                    if matches!(error.class, ConflictClass::StaleTarget { .. }
                        | ConflictClass::DuplicateRelationIdentity { .. })),
                    "unexpected refusal seed {seed} split {split} step {step}: {log:?}: {error:?}"
                );
                tally.other += 1;
            }
        }
    }
}

pub(super) fn random_edits(seeds: std::ops::RangeInclusive<u64>) {
    let mut tally = Tally::default();
    for seed in seeds {
        for split in [false, true] {
            journey(seed, split, 80, &mut tally);
        }
    }
    eprintln!("MIXED_LAW_TALLY {tally:?}");
    assert!(tally.refused_law > 0 && tally.accepted > 0);
    assert!(tally.entity_deletes > 0 && tally.replaces > 0 && tally.same_commit_restores > 0);
    assert!(tally.unrelated > 0 && tally.reopens > 0);
}
