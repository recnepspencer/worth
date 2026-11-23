//! Reopen work follows the checkpoint and its tail, never the history behind it.
use super::cold_checkpoint::ColdCheckpointSource;
use crate::content::historical_kind::fixture::World;
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{
    durability::*, identity::KindId, runtime::RelationalRuntime, transactions::*,
};

/// Repoint one relation; the live state keeps its size while history grows.
fn repoint(world: &World, request: &ExecutionRequest<'_, '_>, step: usize) {
    let update = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: world.relations[0],
            kind_id: KindId(2),
            source: EntityReference::Existing(world.entities[0]),
            target: EntityReference::Existing(world.entities[1 + step % 2]),
        },
    ));
    let committed = world.commit(request, update).unwrap();
    world.release(&committed);
}

/// Counted work of one recovery from a checkpoint after `history` edits, with
/// `tail` further edits replayed from the journal.
fn reopen_work(history: usize, tail: usize) -> u64 {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    for name in ["source", "target-a", "target-b"] {
        world.create(&request, 7, name);
    }
    world.relate(&request, 0, 1, 7, "repointed");
    for step in 0..history {
        repoint(&world, &request, step);
    }
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = ColdCheckpointSource::capture(&world.runtime, &request, &checkpoint);
    let covered = checkpoint.covered_position().unwrap();
    for step in 0..tail {
        repoint(&world, &request, history + step);
    }
    let journal = world
        .runtime
        .durability()
        .canonical_commit_artifacts(&request)
        .map(Result::unwrap)
        .filter(|artifact| artifact.position() > covered)
        .collect::<Vec<_>>();
    assert_eq!(journal.len(), tail);
    let metered = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let receiving = World::new().runtime;
    let (recovered, report) = ExecutionWorkCeiling::new(None)
        .run_serial(&metered, || {
            recover(
                &ExecutionRequest::serial(&metered),
                &checkpoint,
                journal,
                Arc::clone(&source),
                receiving,
            )
        })
        .unwrap();
    let head = |runtime: &RelationalRuntime| {
        let (descriptor, _) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let target = descriptor
            .reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id();
        (target, descriptor.truth_version())
    };
    assert_eq!(
        head(&recovered),
        head(&world.runtime),
        "recovery reaches the live head"
    );
    // Tail replay demands the one region its edits touch; no other region.
    assert_eq!(
        source.region_read_count(),
        1,
        "reopen demands only the region its tail edits"
    );
    report.charged_work()
}

fn recover(
    request: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    tail: Vec<RelationalCanonicalCommitArtifact>,
    source: Arc<ColdCheckpointSource>,
    mut runtime: RelationalRuntime,
) -> RelationalRuntime {
    let replayed = tail.len();
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        )),
        tail,
        vec![],
        source,
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered = runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(recovered.coverage.replayed_tail_commits, replayed);
    runtime
}

#[test]
fn reopen_work_is_flat_in_history_behind_the_checkpoint() {
    // Work is charged per encoded byte, and encoded identities and member
    // lengths vary slightly with their values. Any per-commit reopen work would
    // decode at least one ~30-byte index entry per covered commit, so reopen
    // work that differs by less than one unit per covered commit does none.
    let (short_history, long_history) = (1_000, 10_000);
    let short = reopen_work(short_history, 4);
    let long = reopen_work(long_history, 4);
    eprintln!("reopen work: history {short_history} -> {short}; history {long_history} -> {long}");
    assert!(
        short.abs_diff(long) < (long_history - short_history) as u64,
        "reopen work must not depend on covered history: {short} vs {long}"
    );
}

#[test]
fn reopen_work_grows_linearly_in_the_journal_tail() {
    let costs = [4, 8, 12].map(|tail| reopen_work(10, tail));
    eprintln!("reopen work by tail 4/8/12: {costs:?}");
    assert!(costs[0] < costs[1], "a longer tail costs more");
    assert_eq!(
        costs[1] - costs[0],
        costs[2] - costs[1],
        "equal tail increments cost equal work"
    );
}
