//! Identical actual commits publish the same names/history with real checked workers.
use super::super::read_handoff_rule::{parallel, Control};
use super::fixtures::*;
use crate::aggregate_identity::AggregateRead;
use crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection;
use crate::facade::runtime::{PinnedAggregateLineageScope, PinnedAggregateRead};
use crate::history::data::CommitId;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::facade::{AggregateIdentity, AggregateLineageKind};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

type Event = (
    CommitId,
    AggregateLineageKind,
    AggregateIdentity,
    AggregateIdentity,
);

#[derive(Debug, PartialEq, Eq)]
struct ComponentState {
    definition: AggregateDefinitionIdentity,
    identity: AggregateIdentity,
    members: Vec<EntityId>,
    lineage: Vec<Event>,
}

#[derive(Debug, PartialEq, Eq)]
struct IdentityState {
    definition: AggregateDefinitionIdentity,
    identity: AggregateIdentity,
    retired: Option<CommitId>,
    latest: Option<AggregateIdentity>,
    lineage: Vec<Event>,
    continuation: Vec<Event>,
}

#[derive(Debug, PartialEq, Eq)]
struct PublishedState {
    components: Vec<ComponentState>,
    identities: Vec<IdentityState>,
}

#[test]
fn identical_component_commits_keep_all_names_and_lineage_at_one_two_and_eight_workers() {
    let serial = run(1);
    for workers in [2, 8] {
        assert_eq!(run(workers), serial, "actual {workers}-worker commits");
    }
}

fn run(workers: usize) -> Vec<PublishedState> {
    let registry = registry();
    let control = Arc::new(Control {
        source: registry.relation_kinds[&KindId(7)]
            .relation_integrity
            .aggregate_identity
            .clone(),
        ..Control::default()
    });
    let overlap = parallel::ParallelReads::new(workers);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(parallel::registration(10, &control, &overlap))
        .custom_invariant(parallel::registration(11, &control, &overlap))
        .build()
        .expect("valid native runtime configuration");
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).unwrap(),
                    Some(128 * 1024 * 1024),
                    Some(10_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = &ExecutionRequest::leased(&lease);
    let born = commit(execution, &runtime, birth());
    let ids = ids(&born);
    let mut issued = BTreeSet::new();
    let mut states = vec![published(
        execution,
        &runtime,
        &born.snapshot,
        &ids,
        &mut issued,
    )];
    let edits = [
        WorkerIntentBatch::new("worker-component-split")
            .push(delete_edge(&born, 0, 2))
            .push(delete_edge(&born, 1, 6))
            .push(delete_edge(&born, 5, 6)),
        WorkerIntentBatch::new("worker-component-anchor-removal")
            .push(delete_edge(&born, 2, 6))
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: ids[1] },
            ))),
        WorkerIntentBatch::new("worker-component-merge").push(edge(
            6,
            2,
            EntityReference::Existing(ids[2]),
            EntityReference::Existing(ids[3]),
        )),
    ];
    let mut committed = read(execution, &runtime, &born.snapshot, false, ids[2]);
    for (step, edit) in edits.into_iter().enumerate() {
        control.arm(vec![ids[2]]);
        overlap.reset();
        let result = commit(execution, &runtime, edit);
        if workers > 1 {
            assert!(
                result
                    .validation()
                    .invariant_executions()
                    .iter()
                    .any(|execution| {
                        execution
                            .metadata()
                            .preparation_strategy()
                            .is_some_and(|strategy| {
                                strategy.selected_mode
                                    == PreparationStrategySelection::StagedParallel
                            })
                    }),
                "commit selected real parallel execution"
            );
        }
        overlap.assert_overlap();
        let selected = read(execution, &runtime, &result.snapshot, false, ids[2]);
        assert_captures(&control, &selected, &committed, result.commit.commit_id);
        let live = ids
            .iter()
            .copied()
            .filter(|id| step == 0 || *id != ids[1])
            .collect::<Vec<_>>();
        states.push(published(
            execution,
            &runtime,
            &result.snapshot,
            &live,
            &mut issued,
        ));
        committed = selected;
    }
    assert!(states[1].components.iter().any(|state| state
        .lineage
        .iter()
        .any(|event| event.1 == AggregateLineageKind::SplitFrom)));
    assert!(states[3]
        .identities
        .iter()
        .any(|state| state.retired.is_some()
            && state
                .continuation
                .iter()
                .any(|event| event.1 == AggregateLineageKind::MergedInto)));
    states
}

fn assert_captures(
    control: &Control,
    proposed: &PinnedAggregateRead,
    committed: &PinnedAggregateRead,
    occurrence: CommitId,
) {
    let captures = control.captures.lock().unwrap();
    assert_eq!(
        captures
            .iter()
            .map(|capture| (capture.rule, capture.evaluation))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([(10, false), (10, true), (11, false), (11, true)])
    );
    for capture in captures.iter() {
        assert!(capture.errors.is_empty(), "{:?}", capture.errors);
        assert_eq!(capture.proposed.len(), 1);
        assert_eq!(capture.committed.len(), 1);
        assert_read(&capture.proposed[0], proposed);
        assert_read(&capture.committed[0], committed);
        assert_eq!(
            capture.proposed[0]
                .transition_lineage()
                .map(event)
                .collect::<Vec<_>>(),
            proposed
                .lineage()
                .filter(|event| event.occurrence() == occurrence)
                .map(event)
                .collect::<Vec<_>>()
        );
        assert_eq!(capture.committed[0].transition_lineage().count(), 0);
    }
}

fn assert_read(actual: &AggregateRead, published: &PinnedAggregateRead) {
    assert_eq!(actual.identity(), published.identity());
    assert_eq!(
        actual.members().collect::<Vec<_>>(),
        published.members().collect::<Vec<_>>()
    );
}

fn published(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    live: &[EntityId],
    issued: &mut BTreeSet<(AggregateDefinitionIdentity, AggregateIdentity)>,
) -> PublishedState {
    let mut components = Vec::new();
    for second in [false, true] {
        for &member in live {
            let value = read(execution, runtime, snapshot, second, member);
            issued.insert((value.definition(), value.identity()));
            components.push(ComponentState {
                definition: value.definition(),
                identity: value.identity(),
                members: value.members().collect(),
                lineage: value.lineage().map(event).collect(),
            });
        }
    }
    let pin = runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .unwrap();
    let identities = issued
        .iter()
        .map(|&(definition, identity)| {
            let value = pin
                .read_aggregate_lineage(
                    execution,
                    PinnedAggregateLineageScope::for_definition(definition, identity),
                )
                .unwrap();
            IdentityState {
                definition,
                identity,
                retired: value.retired_at(),
                latest: value.latest_identity(),
                lineage: value.lineage().map(event).collect(),
                continuation: value.resolution_steps().map(event).collect(),
            }
        })
        .collect();
    PublishedState {
        components,
        identities,
    }
}

fn event(value: crate::aggregate_identity::AggregateReadLineageEvent<'_>) -> Event {
    (value.occurrence(), value.kind(), value.from(), value.to())
}
