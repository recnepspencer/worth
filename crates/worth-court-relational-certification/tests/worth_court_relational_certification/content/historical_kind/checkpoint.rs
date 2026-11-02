//! Frozen CDC operations through Native's public records and durability owners.
use super::fixture::{runtime, snapshot, World};
use crate::content::cold_envelope::Source;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};

#[test]
fn retained_relation_history_checkpoint_matches_published_leaf_and_recovers() {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(512 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&original);
    let mut world = World::new();
    for (partition, label) in [
        (7, "left"),
        (11, "right"),
        (29, "center"),
        (7, "four"),
        (7, "five"),
        (29, "six"),
    ] {
        world.create(&execution, partition, label);
    }
    world.replace(&execution, 9, 31, "first-replacement");
    world.relate(&execution, 0, 7, 31, "first-edge");
    world.replace(&execution, 6, 7, "second-replacement");
    world.create(&execution, 7, "seven");
    world.runtime.retention().run_pass(&execution).unwrap();
    world.refresh(&execution);
    world.relate(&execution, 5, 13, 7, "second-edge");
    let retained = snapshot(&world.runtime);
    world.delete_entity(&execution, 3);
    world.delete_relation(&execution, 5);
    world
        .runtime
        .snapshots()
        .release_snapshot(&retained)
        .unwrap();
    // The next two frozen release operations select an empty snapshot vector.
    // They have no Native operation, exactly as in the original regression.
    let checkpoint = capture(&world.runtime, &execution);
    let covered = checkpoint.covered_position().unwrap();
    world.relate(&execution, 1, 7, 31, "third-edge");
    // Independent capture recomputes all root-leaf facts. A cached digest alone
    // must not certify relation-history reuse when incidence journals are empty.
    let final_checkpoint = capture(&world.runtime, &execution);
    let tail = world
        .runtime
        .durability()
        .canonical_commit_artifacts(execution.allocation_policy())
        .map(Result::unwrap)
        .filter(|a| a.position() > covered)
        .map(|a| {
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(a.as_bytes().to_vec()).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(tail.len(), 1);
    let source = Arc::new(Source::capture(
        &world.runtime,
        &execution,
        &checkpoint,
        covered,
        None,
    ));
    let reopened = reopen(&execution, &checkpoint, tail, source);
    // Complete literal identities, generations, kinds, endpoints and values.
    // Neither the serializer nor recovery supplies the expected answer.
    let expected = Truth {
        entities: vec![
            ((7, 0, 1), 1, "left".into()),
            ((7, 1, 1), 1, "four".into()),
            ((7, 2, 1), 1, "five".into()),
            ((7, 4, 1), 1, "seven".into()),
            ((11, 0, 1), 1, "right".into()),
            ((29, 0, 1), 1, "center".into()),
        ],
        relations: vec![((31, 0, 2), 2, (7, 1, 1), (29, 0, 1), "third-edge".into())],
    };
    assert_eq!(truth(&world.runtime, &execution), expected);
    assert_eq!(truth(&reopened, &execution), expected);
    assert_eq!(
        reopened.history().latest_patch_stream_position().unwrap(),
        final_checkpoint.covered_position().unwrap()
    );
}

pub(super) fn reopen(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    tail: Vec<RelationalCanonicalCommitArtifact>,
    source: Arc<Source>,
) -> RelationalRuntime {
    let count = tail.len();
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
    let mut reopened = runtime();
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let outcome = reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(outcome.coverage.replayed_tail_commits, count);
    reopened
}

pub(super) fn capture(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap()
}

type Identity = (u32, u64, u64);
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Truth {
    pub entities: Vec<(Identity, u32, String)>,
    pub relations: Vec<(Identity, u32, Identity, Identity, String)>,
}

pub(super) fn truth(runtime: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) -> Truth {
    let snapshot = snapshot(runtime);
    let pin = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let entities = read
        .entities()
        .iter()
        .map(|e| {
            (
                (
                    e.entity_id.partition_id.0,
                    e.entity_id.slot_index() as u64,
                    e.entity_id.generation.0 as u64,
                ),
                e.kind.kind_id.0,
                text(
                    e.authoritative_aspect_state.as_ref().unwrap(),
                    "name",
                    pin.symbols(),
                ),
            )
        })
        .collect();
    let relations = read
        .relations()
        .iter()
        .map(|r| {
            (
                (
                    r.relation_id.partition_id.0,
                    r.relation_id.slot_index() as u64,
                    r.relation_id.generation.0 as u64,
                ),
                r.kind.kind_id.0,
                (
                    r.source.partition_id.0,
                    r.source.slot_index() as u64,
                    r.source.generation.0 as u64,
                ),
                (
                    r.target.partition_id.0,
                    r.target.slot_index() as u64,
                    r.target.generation.0 as u64,
                ),
                text(
                    r.authoritative_aspect_state.as_ref().unwrap(),
                    "label",
                    pin.symbols(),
                ),
            )
        })
        .collect();
    drop(read);
    drop(pin);
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    Truth {
        entities,
        relations,
    }
}

fn text(
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
    key: &str,
    symbols: &worth_relational::facade::symbols::StringInterner,
) -> String {
    use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValueView};
    let ContractValidatedAspectValueView::Scalar(AspectValue::String(value)) =
        state.get(&AspectKey::new(key).unwrap()).unwrap().view()
    else {
        panic!("fixture has a required scalar text aspect")
    };
    match value {
        worth_foundational::facade::InternedString::Raw(value) => value.clone(),
        worth_foundational::facade::InternedString::Symbol(symbol) => symbols
            .resolve(worth_relational::facade::symbols::Symbol(symbol.0))
            .unwrap()
            .to_owned(),
    }
}
