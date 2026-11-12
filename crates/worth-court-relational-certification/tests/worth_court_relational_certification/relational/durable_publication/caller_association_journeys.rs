//! Caller associations remain exact through proposal, original frames and cold recovery.
use super::branch_history_publications::transaction;
use super::*;
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy, history::BranchId, runtime::RelationalRuntime,
};
fn propose(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> worth_relational::facade::mvcc::ValidatedRelationalProposal {
    let mut transaction = transaction(runtime, &BranchId("main".into()));
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    transaction.validate(execution, runtime).unwrap()
}

fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &TestDurableMedium,
    key: &str,
    association: &Association,
) -> Artifact {
    let proposal = propose(execution, runtime, key)
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    assert_eq!(candidate.caller_association().as_ref(), Some(association));
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the original prepared publication performs")
    };
    assert_eq!(
        performed.canonical_commit().caller_association(),
        Some(association)
    );
    let position = performed.patch_position();
    let receipt = performed.canonical_commit().commit.clone();
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let artifact = medium
        .offered()
        .into_iter()
        .find(|artifact| artifact.position() == position)
        .unwrap();
    let readmitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(readmitted.envelope().commit, receipt);
    assert_eq!(
        readmitted.envelope().semantic_result(),
        committed.semantic_result()
    );
    assert_eq!(
        readmitted.envelope().caller_association(),
        Some(association)
    );
    release_test_commit_snapshot(runtime, &committed);
    artifact
}

#[test]
fn caller_association_refuses_rebinding_and_stays_on_the_prepared_candidate() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let association = Association::from_bytes(b"opaque\0caller\xff").unwrap();
    let proposal = propose(execution, &runtime, "rebound")
        .bind_caller_association(association.clone())
        .unwrap();
    assert!(proposal
        .bind_caller_association(association.clone())
        .is_err());
    let proposal = propose(execution, &runtime, "discarded")
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    assert_eq!(candidate.caller_association().as_ref(), Some(&association));
    drop(candidate);
}

#[test]
fn caller_association_survives_cold_checkpoint_history_and_journal_tail() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let association = Association::from_bytes(b"same-original-caller-attempt").unwrap();
    let first = publish(execution, &runtime, &medium, "cold-history", &association);
    let head = publish(
        execution,
        &runtime,
        &medium,
        "checkpoint-head",
        &association,
    );
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    medium.file_checkpoint(native);
    let tail = publish(execution, &runtime, &medium, "journal-tail", &association);
    let (outcome, recovered) =
        super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium));
    assert_eq!(
        outcome.coverage.checkpoint_covered_position,
        Some(head.position())
    );
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    let actual = recovered
        .durability()
        .canonical_commit_artifacts(execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(actual.len(), 3);
    for (original, recovered) in [first, head, tail].iter().zip(&actual) {
        assert_eq!(recovered.position(), original.position());
        assert_eq!(recovered.commit_id(), original.commit_id());
        assert_eq!(recovered.payload_digest(), original.payload_digest());
        assert_eq!(recovered.as_bytes(), original.as_bytes());
        let readmitted = Artifact::readmit(recovered.as_bytes(), execution).unwrap();
        let original = Artifact::readmit(original.as_bytes(), execution).unwrap();
        assert_eq!(readmitted.envelope().commit, original.envelope().commit);
        assert_eq!(
            readmitted.envelope().semantic_result(),
            original.envelope().semantic_result()
        );
        assert_eq!(
            readmitted.envelope().caller_association(),
            Some(&association)
        );
    }
}
