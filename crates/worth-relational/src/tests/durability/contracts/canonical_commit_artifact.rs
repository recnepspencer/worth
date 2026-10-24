use super::*;
use crate::capabilities::RuntimeConfigSource;
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

fn exported_artifacts(runtime: &RelationalRuntime) -> Vec<Artifact> {
    runtime
        .durability()
        .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .collect::<Result<_, _>>()
        .expect("retained commits encode as artifacts")
}

#[test]
fn every_retained_commit_readmits_from_its_artifact_unchanged() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let runtime = memory_runtime_with_mixed_history(execution);
    assert_eq!(
        runtime.runtime_config().durability.policy.mode,
        DurabilityMode::InMemoryCanonical
    );
    let originals = runtime.history.positioned_canonical_commits_snapshot();
    assert!(originals.len() >= 5, "creates, edit, relation, transition");
    assert!(originals
        .iter()
        .any(|commit| commit.envelope().schema_transition.is_some()));

    let artifacts = exported_artifacts(&runtime);
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

    let artifacts = exported_artifacts(&runtime);
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
