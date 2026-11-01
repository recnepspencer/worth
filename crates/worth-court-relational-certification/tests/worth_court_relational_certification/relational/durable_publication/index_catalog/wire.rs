//! Mutate only the actual producer payload; keep original field order intact.
use super::*;
use sha2::{Digest as _, Sha256};

fn key_offset(body: &[u8], key: &str) -> usize {
    let encoded = rmp_serde::to_vec_named(key).unwrap();
    let found = body
        .windows(encoded.len())
        .enumerate()
        .filter_map(|(n, bytes)| (bytes == encoded).then_some(n))
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1, "one actual field {key}");
    found[0]
}
fn delta_span(body: &[u8]) -> (usize, usize, usize) {
    let key = "index_definition_delta";
    let start = key_offset(body, key);
    let value = start + rmp_serde::to_vec_named(key).unwrap().len();
    (start, value, key_offset(body, "semantic_result"))
}
fn reseal(artifact: &Artifact, body: &[u8]) -> Vec<u8> {
    let mut frame = artifact.as_bytes()[..66].to_vec();
    frame.extend_from_slice(body);
    frame[58..66].copy_from_slice(&(body.len() as u64).to_le_bytes());
    frame[26..58].copy_from_slice(&Sha256::digest(body));
    frame
}
fn replace_delta_scalar(body: &mut [u8], key: &str, old: u8, new: u8) {
    let (_, value, end) = delta_span(body);
    let offset =
        value + key_offset(&body[value..end], key) + rmp_serde::to_vec_named(key).unwrap().len();
    assert_eq!(body[offset], old);
    body[offset] = new;
}
pub(super) fn assert_malformed_deltas(execution: &ExecutionRequest<'_, '_>, artifact: &Artifact) {
    let body = &artifact.as_bytes()[66..];
    assert!(
        Artifact::readmit(&reseal(artifact, body), execution).is_ok(),
        "current resealed original counterpart"
    );
    for malformed in ["missing", "null", "shape", "identity"] {
        let mut changed = body.to_vec();
        let (key, value, end) = delta_span(&changed);
        match malformed {
            "missing" => {
                let canonical = key_offset(body, "canonical")
                    + rmp_serde::to_vec_named("canonical").unwrap().len();
                assert_eq!(changed[canonical], 0xde, "actual canonical map16 header");
                let count =
                    u16::from_be_bytes(changed[canonical + 1..canonical + 3].try_into().unwrap());
                changed[canonical + 1..canonical + 3].copy_from_slice(&(count - 1).to_be_bytes());
                changed.drain(key..end);
            }
            "null" => {
                changed.splice(value..end, [0xc0]);
            }
            "shape" => replace_delta_scalar(&mut changed, "base_next_index_id", 1, 2),
            _ => replace_delta_scalar(&mut changed, "index_id", 1, 9),
        }
        let frame = reseal(artifact, &changed);
        assert!(Artifact::from_untrusted_bytes(frame.clone()).is_ok());
        assert!(matches!(Artifact::readmit(&frame, execution),
            Err(worth_relational::facade::durability::RelationalCanonicalCommitArtifactRefusal::PayloadDecode { .. })),
            "{malformed} must fail required-field/shape decoding before canonical-form checks");
    }
}

#[test]
fn valid_delta_shape_still_refuses_source_identity_gap_and_conflicting_overlap() {
    use worth_relational::facade::durability::{
        RecoveryFailureClass, RelationalDurableRecoverySource, RelationalRecoveryStop,
    };
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    for conflict in [false, true] {
        let medium = TestDurableMedium::new();
        let runtime = durable_runtime(&medium);
        if conflict {
            create_entity(execution, &runtime, "checkpoint-seed");
        }
        let installed = runtime
            .index_authority()
            .register(definition("original-catalog"))
            .unwrap();
        if conflict {
            medium.file_checkpoint(capture(execution, &runtime));
        }
        let result = create_entity_outcome(execution, &runtime, "actual-tail");
        let artifact = medium.offered().pop().unwrap();
        let mut changed = artifact.as_bytes()[66..].to_vec();
        if conflict {
            let (_, value, end) = delta_span(&changed);
            let offset = value
                + key_offset(&changed[value..end], "name")
                + rmp_serde::to_vec_named("name").unwrap().len();
            let encoded = rmp_serde::to_vec_named(&installed.name).unwrap();
            assert_eq!(&changed[offset..offset + encoded.len()], &encoded);
            changed[offset + encoded.len() - 1] ^= 1;
        } else {
            replace_delta_scalar(&mut changed, "base_next_index_id", 1, 2);
            replace_delta_scalar(&mut changed, "next_index_id", 2, 3);
            replace_delta_scalar(&mut changed, "index_id", 1, 2);
        }
        let bad = reseal(&artifact, &changed);
        Artifact::readmit(&bad, execution)
            .expect("negative payload is a current, canonical, shape-valid artifact");
        let actual = medium.recovery_source();
        // Negative-only perturbation of the original producer's stream; the
        // receiving owner must refuse before it can become catalog authority.
        let source = RelationalDurableRecoverySource::new(
            actual.checkpoint().cloned(),
            vec![Artifact::from_untrusted_bytes(bad).unwrap()],
            actual.transitions().to_vec(),
            medium.clone(),
        );
        let mut receiving = durable_runtime(&medium);
        let plan = receiving
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let refusal = receiving
            .durability_recovery()
            .recover_with_authority(execution, plan)
            .unwrap_err();
        assert!(
            matches!(refusal, RelationalRecoveryStop::Durability(ref error)
            if error.class == RecoveryFailureClass::CorruptCanonicalArtifact &&
                error.detail.contains(if conflict { "conflicts" } else { "gap" })),
            "{refusal:?}"
        );
        release_test_commit_snapshot(&runtime, &result);
    }
}
