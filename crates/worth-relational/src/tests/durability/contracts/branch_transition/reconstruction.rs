//! Replay and recovery reconstruct only the history absent from their basis.
use super::*;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurablePublication};
use crate::facade::durability::RecoveryVerificationMode;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};
use crate::facade::runtime::RelationalRuntimeApi;
use std::sync::{Arc, Mutex};

fn replay(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: crate::history::data::CommitId,
    branch: &str,
) {
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: commit,
                branch_id: BranchId(branch.into()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(replay.mismatches.is_empty(), "{replay:?}");
}

#[test]
fn memory_checkpoint_keeps_replay_reconstructible() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema_profile(
        crate::config::data::RelationalRuntimeProfile::GeometryKernel,
    );
    assert_eq!(
        runtime.config().durability.policy.log.retention_mode,
        crate::config::data::DurableLogRetentionMode::CompactAfterCheckpoint
    );
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let target = create_entity_outcome(execution, &runtime, "after-checkpoint");
    replay(execution, &runtime, target.commit.commit_id, "main");
    // This earlier chain cannot select the later checkpoint: memory has no
    // journal from which to restore its covered fork.
    let on_a = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    replay(execution, &runtime, target.commit.commit_id, "main");
    replay(execution, &runtime, on_a.commit.commit_id, "A");
}

#[derive(Debug)]
struct CountedTransitionReads {
    medium: Box<dyn RelationalDurableMedium>,
    reads: Arc<Mutex<Vec<Sequence>>>,
}

impl RelationalDurableMedium for CountedTransitionReads {
    fn read_transition(
        &self,
        sequence: Sequence,
    ) -> Result<Artifact, crate::durability::medium::RelationalDurableTransitionReadFailure> {
        self.reads.lock().unwrap().push(sequence);
        self.medium.read_transition(sequence)
    }
    fn publish(
        &self,
        artifact: &crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        self.medium.publish(artifact)
    }
    fn publish_transition(&self, artifact: &Artifact) -> RelationalDurablePublication {
        self.medium.publish_transition(artifact)
    }
}

#[test]
fn replay_reads_only_the_gap_after_its_checkpoint() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    for covered in [1, 32] {
        let contents = TestDurableMedium::new();
        let reads = Arc::new(Mutex::new(Vec::new()));
        let runtime = RelationalRuntimeApi::builder()
            .profile(crate::config::data::RelationalRuntimeProfile::CertificationCore)
            .schema_registry(test_schema_registry())
            .build_durable(Box::new(CountedTransitionReads {
                medium: contents.attachment(),
                reads: Arc::clone(&reads),
            }));
        create_entity(execution, &runtime, "source");
        for index in 0..covered {
            fork(execution, &runtime, &format!("covered-{index}"));
        }
        runtime
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        for name in ["gap-1", "gap-2"] {
            fork(execution, &runtime, name);
        }
        runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap();
        let target = create_entity_outcome_on_branch(
            execution,
            &runtime,
            "target",
            BranchId("gap-2".into()),
        );
        reads.lock().unwrap().clear();
        replay(execution, &runtime, target.commit.commit_id, "gap-2");
        let reads = reads.lock().unwrap();
        assert!(reads.len() <= 2, "covered history was reread: {reads:?}");
        assert!(
            reads
                .iter()
                .all(|sequence| sequence.0 > covered && sequence.0 <= covered + 2),
            "read outside the missing gap: {reads:?}"
        );
    }
}

#[test]
fn recovery_plan_after_native_capture_keeps_covered_forks() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("A".into()))
            .unwrap()
            .commit_id,
        target.commit.commit_id
    );
}

#[test]
fn a_memory_fork_can_replay_its_compacted_durable_source() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let speculative = runtime.fork().unwrap();
    assert!(
        speculative.durability.medium().is_none(),
        "a runtime fork cannot publish to its source's medium"
    );
    replay(execution, &speculative, target.commit.commit_id, "A");
}
