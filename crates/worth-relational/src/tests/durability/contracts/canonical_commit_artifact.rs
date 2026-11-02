use super::*;
use crate::facade::durability::RelationalCanonicalCommitArtifact as Artifact;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};

// CONTRACT: canonical commit artifact
// LANES: success, determinism, ordering, refusal

mod refusals;

/// A memory-only runtime whose retained history spans record creates, an
/// edit, a relation and a schema transition, all produced by real commits.
fn memory_runtime_with_mixed_history(execution: &ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let mut runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "artifact-source");
    let target = create_entity(execution, &runtime, "artifact-target");
    update_entity_and_release_snapshot(execution, &runtime, source, "artifact-source-renamed");
    let relation = create_relation_outcome(execution, &runtime, source, target, "artifact-link");
    release_test_commit_snapshot(&runtime, &relation);
    commit_schema_transition(execution, &mut runtime);
    runtime
}

fn commit_schema_transition(execution: &ExecutionRequest<'_, '_>, runtime: &mut RelationalRuntime) {
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let input = crate::tests::support::test_owner_transaction_validation_input_for_main(runtime)
        .with_schema_transition(
            schema_transition_for_subscriber_impact(
                SchemaVersionId(2),
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
            ),
            Some(SchemaReconciliationPolicy::PreserveInformation),
        );
    let mut txn = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .expect("owner-admitted transaction context");
    txn.push_batch(
        batch_create("artifact-after-transition"),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let transitioned = txn.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &transitioned);
}

fn exported_artifacts(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<Artifact> {
    runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .collect::<Result<_, _>>()
        .expect("retained commits encode as artifacts")
}

#[test]
fn every_retained_commit_readmits_from_its_artifact_unchanged() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let runtime = memory_runtime_with_mixed_history(execution);
    assert!(
        runtime.durability.medium().is_none(),
        "the artifacts come from a memory-only runtime's own history"
    );
    let originals = runtime.history.positioned_canonical_commits_snapshot();
    assert!(originals.len() >= 5, "creates, edit, relation, transition");
    assert!(originals
        .iter()
        .any(|commit| commit.envelope().schema_transition.is_some()));

    let artifacts = exported_artifacts(execution, &runtime);
    assert_eq!(artifacts.len(), originals.len());
    for (original, artifact) in originals.iter().zip(&artifacts) {
        let original_envelope = original.envelope();
        assert_eq!(artifact.position(), original.position());
        assert_eq!(artifact.commit_id(), original_envelope.commit.commit_id);

        let readmitted = Artifact::readmit(artifact.as_bytes(), execution)
            .expect("an exported artifact readmits");
        let envelope = readmitted.envelope();
        assert_eq!(readmitted.position(), original.position());
        assert_eq!(readmitted.commit_id(), original_envelope.commit.commit_id);
        assert_eq!(
            envelope.encode_authoritative_payload().unwrap(),
            original_envelope.encode_authoritative_payload().unwrap()
        );
        assert_eq!(envelope.symbol_texts, original_envelope.symbol_texts);
        assert_eq!(
            envelope.required_output_codec_retirements,
            original_envelope.required_output_codec_retirements
        );
    }
}

#[test]
fn encoding_a_commit_is_deterministic_across_encodes_and_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let runtime = memory_runtime_with_mixed_history(execution);

    for original in runtime.history.positioned_canonical_commits_snapshot() {
        let first =
            Artifact::from_positioned(&original, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
        let second =
            Artifact::from_positioned(&original, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());

        let readmitted = Artifact::readmit(first.as_bytes(), execution).unwrap();
        let reencoded = Artifact::from_positioned(
            readmitted.positioned(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        assert_eq!(reencoded.as_bytes(), first.as_bytes());
    }
}

#[test]
fn retained_artifacts_follow_strictly_increasing_positions() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let runtime = memory_runtime_with_mixed_history(execution);

    let artifacts = exported_artifacts(execution, &runtime);
    let positions = artifacts.iter().map(Artifact::position).collect::<Vec<_>>();
    assert!(positions.len() >= 5);
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

    for artifact in &artifacts {
        let reframed = Artifact::from_untrusted_bytes(artifact.as_bytes().to_vec())
            .expect("exported bytes carry an intact frame");
        assert_eq!(reframed.position(), artifact.position());
        assert_eq!(reframed.commit_id(), artifact.commit_id());
        assert_eq!(reframed.payload_digest(), artifact.payload_digest());
    }
}

#[test]
fn positioned_agreement_uses_original_work_budget_and_preserves_authoritative_digest() {
    use crate::durability::data::RelationalRecoveryStop as Stop;
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use worth_execution::{CancellationSource, SerialRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&setup_execution, &runtime, &"x".repeat(192 * 1024));
    let original = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    let artifact = exported_artifacts(&setup_execution, &runtime)
        .pop()
        .unwrap();
    let bytes_before = artifact.as_bytes().to_vec();
    let readmitted = artifact.readmit_owned(&setup_execution).unwrap();
    let candidate = readmitted.positioned().canonical_arc();
    assert!(!std::sync::Arc::ptr_eq(original.canonical_arc(), candidate));
    let baseline = Artifact::matches_positioned_payload(&original, candidate).unwrap();
    assert!(baseline);
    assert_eq!(
        Artifact::matches_positioned_payload_for_request(&original, candidate, &setup_execution,)
            .unwrap(),
        baseline
    );
    assert_eq!(
        Artifact::envelope_payload_digest_for_request(
            original.position(),
            original.envelope(),
            &setup_execution,
        )
        .unwrap(),
        *artifact.payload_digest().as_bytes()
    );
    // A second genuine publication must not agree with the first admitted
    // body, even when the implementation still debits the full work.
    let other = create_entity_outcome(&setup_execution, &runtime, "different-published-body");
    let other_original = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    assert!(!Artifact::matches_positioned_payload(&other_original, candidate).unwrap());
    assert!(!Artifact::matches_positioned_payload_for_request(
        &other_original,
        candidate,
        &setup_execution,
    )
    .unwrap());
    // Actual verified v5 wire: the commitment hashes the three VALUES, without
    // their outer map header/keys. Count exact original slices independently.
    let payload_len = u64::from_le_bytes(artifact.as_bytes()[58..66].try_into().unwrap());
    assert_eq!(artifact.as_bytes().len() as u64, 66 + payload_len);
    assert!(payload_len > 64 * 1024);
    let value_bytes = commitment_value_bytes(artifact.as_bytes());
    assert!(value_bytes > 64 * 1024 && value_bytes < payload_len);
    let policy = |work| {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(64 << 20), Some(work)),
        )
    };
    // Entry/controller charges no payload work. This ceiling admits the whole
    // first hash and all but the final byte of the second, so entry-only polls
    // or a fresh/reset second-pass budget cannot make this test pass.
    let two_pass_work = value_bytes.checked_mul(2).unwrap() - 1;
    let limited = SerialRequest::from_policy(
        &policy(two_pass_work),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&limited);
    assert_eq!(execution.work_ceiling(), Some(two_pass_work));
    assert_eq!(
        Artifact::matches_positioned_payload_for_request(&original, candidate, &execution,)
            .unwrap_err(),
        Stop::Request(Cause::WorkExhausted)
    );
    assert_eq!(limited.memory().unwrap().charged_memory_bytes(), 0);
    // The cold retained-entry digest door also stops INSIDE its one pass.
    let cold = SerialRequest::from_policy(
        &policy(value_bytes - 1),
        CancellationSource::new().token(),
        None,
    );
    assert_eq!(
        Artifact::envelope_payload_digest_for_request(
            original.position(),
            original.envelope(),
            &ExecutionRequest::serial(&cold),
        )
        .unwrap_err(),
        Stop::Request(Cause::WorkExhausted)
    );
    assert_eq!(cold.memory().unwrap().charged_memory_bytes(), 0);
    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let stopped = SerialRequest::from_policy(&policy(two_pass_work), cancelled.token(), None);
    assert_eq!(
        Artifact::matches_positioned_payload_for_request(
            &original,
            original.canonical_arc(),
            &ExecutionRequest::serial(&stopped),
        )
        .unwrap_err(),
        Stop::Request(Cause::Cancelled)
    );
    let expired = SerialRequest::from_policy(
        &policy(two_pass_work),
        CancellationSource::new().token(),
        Some(std::time::Instant::now()),
    );
    assert_eq!(
        Artifact::matches_positioned_payload_for_request(
            &original,
            original.canonical_arc(),
            &ExecutionRequest::serial(&expired),
        )
        .unwrap_err(),
        Stop::Request(Cause::DeadlineElapsed)
    );
    assert_eq!(artifact.as_bytes(), bytes_before);
    assert_eq!(stopped.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(expired.memory().unwrap().charged_memory_bytes(), 0);
    release_test_commit_snapshot(&runtime, &other);
    release_test_commit_snapshot(&runtime, &created);
}

fn commitment_value_bytes(frame: &[u8]) -> u64 {
    let payload = &frame[66..];
    assert_eq!(payload[0], 0x83, "v5 has exactly three named fields");
    let mut cursor = std::io::Cursor::new(payload);
    cursor.set_position(1);
    let mut bytes = 0;
    for expected in ["position", "canonical", "original_inputs"] {
        let key: String = rmp_serde::from_read(&mut cursor).unwrap();
        assert_eq!(key, expected);
        let start = cursor.position();
        let _: serde::de::IgnoredAny = rmp_serde::from_read(&mut cursor).unwrap();
        bytes += cursor.position() - start;
    }
    assert_eq!(cursor.position() as usize, payload.len());
    bytes
}
