//! Replay reads only the branch-transition gap after its checkpoint.
//! Restored from f9f15bf8c8:tests/durability/contracts/branch_transition/reconstruction.rs
//! over TestDurableMedium.
use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch, fork};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence as Sequence,
    config::RelationalRuntimeProfile,
    durability::{
        DurablePublicationFate, RelationalBranchTransitionArtifact,
        RelationalCanonicalCommitArtifact, RelationalDurableColdSource, RelationalDurableMedium,
        RelationalDurableTransitionReadFailure,
    },
    history::{BranchId, CommitId},
    replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::schema,
};

#[derive(Debug)]
struct CountedTransitionReads {
    medium: Box<dyn RelationalDurableMedium>,
    reads: Arc<Mutex<Vec<Sequence>>>,
}

impl RelationalDurableMedium for CountedTransitionReads {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.medium.cold_source()
    }
    fn read_transition(
        &self,
        sequence: Sequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        self.reads.lock().unwrap().push(sequence);
        self.medium.read_transition(sequence)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        self.medium.publish(artifact)
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.medium.publish_transition(artifact)
    }
}

fn replay(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: CommitId,
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
fn replay_reads_only_the_gap_after_its_checkpoint() {
    let request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(
        );
    let execution = &ExecutionRequest::serial(&request);
    for covered in [1u64, 32] {
        let contents = TestDurableMedium::new();
        let reads = Arc::new(Mutex::new(Vec::new()));
        let runtime = RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(schema())
            .build_durable(Box::new(CountedTransitionReads {
                medium: contents.attachment(),
                reads: Arc::clone(&reads),
            }))
            .expect("valid native runtime configuration");
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
            .native_checkpoint(execution, execution.allocation_policy())
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
