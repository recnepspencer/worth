//! Association wire meaning and embedded original encoding are owner value units.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
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
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
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
fn caller_association_value_transport_rejects_invalid_lengths_and_wire() {
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
}

#[test]
fn caller_association_embedded_checkpoint_codec_preserves_original_bytes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
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
}
