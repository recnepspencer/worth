//! Caller bytes cross the actual Native proposal, artifact and cold recovery owners.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::durability::data::RelationalCheckpointArtifactPolicy;
use crate::facade::mvcc::{
    RelationalCallerAssociation as Association, RelationalPublicationOutcome,
};
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

fn propose(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> crate::mvcc::ValidatedRelationalProposal {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    transaction.validate(execution, runtime).unwrap()
}

fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
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
    let artifact = attached_test_medium(runtime)
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
fn caller_association_bounds_and_rebinding_refuse_without_retaining_publication() {
    use crate::facade::mvcc::RelationalCallerAssociationRefusal as Refusal;
    fn transport<T: Send + Sync + 'static>() {}
    transport::<Association>();
    assert_eq!(Association::from_bytes(&[]), Err(Refusal::Empty));
    let full = Association::from_bytes(&[0xff; Association::MAX_LEN]).unwrap();
    assert_eq!(full.as_bytes().len(), Association::MAX_LEN);
    let encoded = rmp_serde::to_vec_named(&full).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<Association>(&encoded).unwrap(),
        full
    );
    assert!(rmp_serde::from_slice::<Association>(&[0xc4, 0]).is_err());
    let mut oversized_wire = vec![0xc5, 0x10, 0x01];
    oversized_wire.extend_from_slice(&[0; Association::MAX_LEN + 1]);
    assert!(rmp_serde::from_slice::<Association>(&oversized_wire).is_err());
    assert_eq!(
        Association::from_bytes(&[0; Association::MAX_LEN + 1]),
        Err(Refusal::TooLong {
            actual: Association::MAX_LEN + 1
        })
    );
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let routes = runtime.history.pending_canonical_publication_route_count();
    let association = Association::from_bytes(b"opaque\0caller\xff").unwrap();
    let proposal = propose(execution, &runtime, "rebound")
        .bind_caller_association(association.clone())
        .unwrap();
    assert!(proposal
        .bind_caller_association(association.clone())
        .is_err());
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    let proposal = propose(execution, &runtime, "discarded")
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    assert_eq!(candidate.caller_association().as_ref(), Some(&association));
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        routes + 1
    );
    drop(candidate);
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        routes
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
}

#[test]
fn caller_association_survives_embedded_checkpoint_and_refuses_old_semantics() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let association = Association::from_bytes(b"embedded-checkpoint-owner").unwrap();
    let artifact = publish(execution, &runtime, "embedded", &association);
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        native.bytes(),
    )
    .unwrap();
    let commit = decoded
        .checkpoint
        .envelopes
        .iter()
        .find(|commit| commit.position() == artifact.position())
        .unwrap();
    assert_eq!(commit.envelope().caller_association(), Some(&association));
    let reencoded = Artifact::from_positioned(commit, execution.allocation_policy()).unwrap();
    assert_eq!(reencoded.as_bytes(), artifact.as_bytes());
    for prefix in [b"WORTH-REL-CHK13\n", b"WORTH-REL-CHK14\n"] {
        let mut old = native.bytes().to_vec();
        old[..prefix.len()].copy_from_slice(prefix);
        let stop =
            crate::durability::log::native_checkpoint_codec::decode_checkpoint(execution, &old)
                .unwrap_err();
        assert_eq!(
            expect_durability_failure(stop).class,
            crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics
        );
    }
}

#[test]
fn caller_association_survives_cold_checkpoint_history_and_journal_tail() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let association = Association::from_bytes(b"same-original-caller-attempt").unwrap();
    let first = publish(execution, &runtime, "cold-history", &association);
    let head = publish(execution, &runtime, "checkpoint-head", &association);
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
    attached_test_medium(&runtime).file_checkpoint(native);
    let tail = publish(execution, &runtime, "journal-tail", &association);
    let (outcome, recovered) = recover_with(execution, &runtime, durable_runtime_with_test_schema);
    assert_eq!(
        outcome.coverage.checkpoint_covered_position,
        Some(head.position())
    );
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    let cold = recovered
        .history
        .commit_artifact(first.commit_id())
        .unwrap()
        .envelope_residency();
    assert!(cold.journal_placement().is_some());
    assert!(
        cold.positioned_if_resident().is_none(),
        "historical body begins cold"
    );
    let actual = recovered
        .durability()
        .canonical_commit_artifacts(&execution)
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
