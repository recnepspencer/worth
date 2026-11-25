//! Only the internal codec observes whether a decoded entry carries live verification.
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;

#[test]
fn authored_index_checkpoint_decoding_does_not_mint_live_verification() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let source = test_owner_main_basis(&runtime).unwrap();
    let definition = register(&runtime, name_locator());
    let built = runtime.index_authority().build_for_basis(
        &execution,
        crate::facade::indexes::DerivedIndexBuildRequest {
            source_commit_id: source.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &source,
    );
    assert!(built.failed_indexes.is_empty());
    create_branch_from_main(&runtime, "index-source");
    let equal = update_entity(&execution, &runtime, entity, "before");
    let envelope = runtime
        .history
        .canonical_envelope(equal.commit.commit_id)
        .unwrap();
    let encoded = rmp_serde::to_vec_named(envelope.as_ref()).unwrap();
    let decoded: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&encoded).unwrap();
    assert_eq!(decoded, *envelope);
    assert!((*decoded.owner_authored_index_writes())
        .verified_entries()
        .is_none());
    // This owned decoded description has no live verification. Form the
    // optional-field-absent legacy wire without JSON's bounded Value numbers;
    // it is a codec fixture, not newly admitted historical evidence.
    let mut old_shape = decoded;
    (*old_shape
        .body_mut_for_test()
        .owner_authored_index_writes_mut()) = Default::default();
    let legacy_bytes = rmp_serde::to_vec_named(&old_shape).unwrap();
    let legacy: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&legacy_bytes).unwrap();
    assert!((*legacy.owner_authored_index_writes()).is_unavailable());
    runtime
        .durability_authority()
        .checkpoint(&execution)
        .unwrap();
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(
            &execution,
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_expanded_checkpoint(
        &execution,
        complete.checkpoint().bytes(),
        crate::tests::support::durability::captured_members_source(complete.checkpoint()),
    )
    .unwrap()
    .checkpoint;
    let decoded_equal = decoded
        .envelopes()
        .iter()
        .find(|entry| entry.envelope().commit.commit_id == equal.commit.commit_id)
        .expect("actual emitted checkpoint contains the authored equal-value occurrence");
    assert!((*decoded_equal.envelope().owner_authored_index_writes())
        .verified_entries()
        .is_none());
    let mut plan = runtime
        .durability()
        .recovery_plan(
            &execution,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint = Some(decoded);
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap();
    assert!((*recovered
        .history
        .canonical_envelope(equal.commit.commit_id)
        .unwrap()
        .owner_authored_index_writes())
    .verified_entries()
    .is_none());
    for outcome in [&created, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
