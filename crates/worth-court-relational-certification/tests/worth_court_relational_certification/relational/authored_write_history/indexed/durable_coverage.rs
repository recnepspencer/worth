//! Public #384: index-keyed history across a reopen is certified by the
//! receiving catalog owner. Every split of one durable history between a
//! checkpoint and its WAL tail answers identically; a legacy packet stays
//! unprovenanced on both paths.
use super::super::{admit_main, name_locator, selected};
use super::membership::key;
use super::{
    changed_entities, create_entity_outcome, fork, release_test_commit_snapshot, scan,
    serial_request, update_entity, BranchId, ExecutionRequest,
};
use crate::messagepack_value_spans::field;
use std::sync::Arc;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    durability::{
        RecoveryVerificationMode, RelationalCanonicalCommitArtifact as Artifact,
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
        RelationalNativeCheckpoint,
    },
    history::{CommitId, RelationalAuthoredWriteDenial},
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{capture, durable_runtime, recover},
};

#[path = "durable_coverage/trace.rs"]
mod trace;
use trace::Trace;

fn register(runtime: &RelationalRuntime, name: &str) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: name.into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: name_locator(),
            },
            branch_scoped: true,
            unique_entity_kind: None,
        })
        .unwrap()
}

fn build(execution: ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, id: DerivedIndexId) {
    let source = admit_main(runtime).unwrap();
    let built = runtime.index_authority().build_for_basis(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: source.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: vec![id],
        },
        &source,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
}

fn branch(runtime: &RelationalRuntime, name: &str) -> AdmittedRelationalBranchBasis {
    runtime
        .admit_branch_basis(&runtime.branch_identity(&BranchId(name.into())).unwrap())
        .unwrap()
}

/// One durable history: a commit before the index exists, then two after it,
/// with a checkpoint captured between them and one at the end. Forks mark
/// the spans: `pre-install` from commit 2, `covered-source` from commit 3 and
/// `tail-source` from commit 4.
struct History {
    medium: Arc<TestDurableMedium>,
    commits: [CommitId; 4],
    definition: DerivedIndexDefinition,
    covered: Vec<CommitId>,
    mid: RelationalNativeCheckpoint,
    end: RelationalNativeCheckpoint,
}

fn history(execution: ExecutionRequest<'_, '_>) -> History {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    fork(&execution, &runtime, "pre-install");
    let uncovered = update_entity(&execution, &runtime, entity, "before");
    let definition = register(&runtime, "covered");
    build(execution, &runtime, definition.index_id);
    fork(&execution, &runtime, "covered-source");
    let first = update_entity(&execution, &runtime, entity, "before");
    let mid = capture(&execution, &runtime);
    fork(&execution, &runtime, "tail-source");
    let second = update_entity(&execution, &runtime, entity, "before");
    let end = capture(&execution, &runtime);
    let covered = vec![first.commit.commit_id, second.commit.commit_id];
    let commits = [&created, &uncovered, &first, &second].map(|outcome| outcome.commit.commit_id);
    for outcome in [&created, &uncovered, &first, &second] {
        release_test_commit_snapshot(&runtime, outcome);
    }
    History {
        medium,
        commits,
        definition,
        covered,
        mid,
        end,
    }
}

/// Rebuild the recovered head's authored-write generation from decoded history.
fn rebuild(execution: ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) {
    let target = admit_main(runtime).unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(runtime, &target))
        .unwrap();
}

fn checkpoint_source(checkpoint: &RelationalNativeCheckpoint) -> RelationalDurableCheckpointSource {
    RelationalDurableCheckpointSource::new(
        checkpoint.covered_position().unwrap(),
        checkpoint.covered_branch_transitions().unwrap(),
        RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
    )
}

/// Recovery from an explicit split: `checkpoint` (if any) plus every later artifact.
fn recover_split(
    execution: ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
    checkpoint: Option<&RelationalNativeCheckpoint>,
) -> RelationalRuntime {
    if let Some(checkpoint) = checkpoint {
        medium.file_checkpoint(checkpoint.clone());
    }
    let checkpoint = checkpoint.map(checkpoint_source);
    let (covered, transitions) = checkpoint
        .as_ref()
        .map_or((0, 0), |source| (source.covered().0, source.covered_transitions().0));
    let source = RelationalDurableRecoverySource::new(
        checkpoint,
        medium
            .committed_artifacts()
            .unwrap()
            .into_iter()
            .filter(|artifact| artifact.position().0 > covered)
            .collect(),
        medium
            .offered_transitions()
            .into_iter()
            .filter(|transition| transition.sequence().0 > transitions)
            .collect(),
        medium.clone(),
    );
    let mut runtime = durable_runtime(medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            &execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover_with_authority(&execution, plan)
        .unwrap_or_else(|stop| panic!("the split recovers: {stop:?}"));
    runtime
}

/// What a recovered runtime answers for the covered index and an index
/// registered after the reopen, over spans from before and after installation.
#[derive(Debug, PartialEq)]
struct Answers {
    covered: Vec<CommitId>,
    from_pre_install: Answer,
    registered_after_reopen: Answer,
}

/// A cursor answer, comparable across recovered runtimes.
type Answer = Result<(), String>;

fn incomplete() -> Answer {
    Err(format!(
        "{:?}",
        RelationalAuthoredWriteDenial::IncompleteIndexHistory
    ))
}

fn cursor(
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: &str,
    definition: &DerivedIndexDefinition,
) -> Answer {
    let target = admit_main(runtime).unwrap();
    runtime
        .authored_write_history_cursor(
            execution,
            &branch(runtime, source),
            selected(runtime, &target),
            &[key(definition, "before")],
        )
        .map(drop)
        .map_err(|denial| format!("{denial:?}"))
}

/// The answers and the id of the index registered after the reopen.
fn answers(
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    definition: &DerivedIndexDefinition,
) -> (Answers, DerivedIndexId) {
    let later = register(runtime, "registered-after-reopen");
    rebuild(execution, runtime);
    let target = admit_main(runtime).unwrap();
    let answers = Answers {
        covered: scan(
            runtime,
            execution,
            &branch(runtime, "covered-source"),
            &target,
            key(definition, "before"),
        )
        .0,
        from_pre_install: cursor(execution, runtime, "pre-install", definition),
        registered_after_reopen: cursor(execution, runtime, "covered-source", &later),
    };
    (answers, later.index_id)
}

#[test]
fn every_checkpoint_and_wal_split_certifies_the_same_index_history() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let history = history(execution);
    let expected = Answers {
        covered: history.covered.clone(),
        from_pre_install: incomplete(),
        registered_after_reopen: incomplete(),
    };
    let covered = history.definition.index_id;
    for (view, checkpoint, checkpointed) in [
        ("all-checkpoint", Some(&history.end), 4),
        ("wal-only", None, 0),
        ("mid-checkpoint-and-tail", Some(&history.mid), 3),
    ] {
        let runtime = recover_split(execution, &history.medium, checkpoint);
        let (answers, later) = answers(execution, &runtime, &history.definition);
        Trace::recovered(&history, checkpointed)
            .register()
            .postings(covered, 3, &answers.covered)
            .admit(covered, 2, &answers.from_pre_install)
            .admit(later, 3, &answers.registered_after_reopen)
            .write(view);
        assert_eq!(
            answers, expected,
            "{view}: certified decoded and replayed commits answer alike"
        );
    }
}

/// Rewrite one artifact's index-write packet into the legacy `Complete` form
/// that predates the persisted coverage bound, keeping its entries.
fn legacy(artifact: Artifact) -> Artifact {
    let bytes = artifact.as_bytes();
    let body = &bytes[66..];
    let packet = field(field(body, "canonical"), "authored_index_writes");
    let entries = match field_name(packet) {
        "Complete" => return artifact,
        "Covered" => field(field(packet, "Covered"), "entries"),
        other => panic!("unexpected packet {other}"),
    };
    let mut replacement = vec![0x81];
    replacement.extend(rmp_serde::to_vec("Complete").unwrap());
    replacement.extend_from_slice(entries);
    let start = packet.as_ptr() as usize - body.as_ptr() as usize;
    let mut changed = body.to_vec();
    changed.splice(start..start + packet.len(), replacement);
    let mut frame = bytes[..66].to_vec();
    frame.extend_from_slice(&changed);
    crate::canonical_commit_frame::reseal(&mut frame);
    Artifact::from_untrusted_bytes(frame).unwrap()
}

fn field_name(packet: &[u8]) -> &str {
    assert_eq!(packet[0], 0x81, "externally tagged packet variant");
    let name_len = match packet[1] {
        marker @ 0xa0..=0xbf => (marker & 31) as usize,
        marker => panic!("short variant name, got {marker:#x}"),
    };
    std::str::from_utf8(&packet[2..2 + name_len]).unwrap()
}

#[test]
fn a_wal_tail_legacy_packet_refuses_like_the_same_packet_from_a_checkpoint() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let history = history(execution);
    let last = *history.covered.last().unwrap();
    let artifacts = history.medium.committed_artifacts().unwrap();
    let artifact = artifacts.into_iter().find(|artifact| artifact.commit_id() == last);
    history.medium.replace_artifact(legacy(artifact.unwrap()));
    let replayed = recover_split(execution, &history.medium, Some(&history.mid));
    rebuild(execution, &replayed);
    let wal_tail = cursor(execution, &replayed, "tail-source", &history.definition);
    let covered = history.definition.index_id;
    Trace::recovered(&history, 3)
        .admit(covered, 4, &wal_tail)
        .write("legacy-wal-tail");
    assert_eq!(
        wal_tail,
        incomplete(),
        "a replayed legacy packet is never re-minted as covered"
    );
    // Replay keeps the decoded legacy packet unchanged, so a checkpoint of a
    // whole-WAL replay carries the same unprovenanced commit.
    let whole_wal = recover_split(execution, &history.medium, None);
    let checkpoint = capture(&execution, &whole_wal);
    drop((replayed, whole_wal));
    history.medium.file_checkpoint(checkpoint);
    let (restored, _) = recover(&execution, &history.medium);
    rebuild(execution, &restored);
    let checkpointed = cursor(execution, &restored, "tail-source", &history.definition);
    Trace::recovered(&history, 4)
        .admit(covered, 4, &checkpointed)
        .write("legacy-checkpoint");
    assert_eq!(checkpointed, wal_tail, "checkpoint and WAL tail agree");
}
