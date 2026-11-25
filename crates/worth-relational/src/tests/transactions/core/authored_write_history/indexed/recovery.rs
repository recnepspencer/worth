//! Only the internal codec observes whether a decoded entry carries live
//! verification, and only the receiving catalog certifies a persisted bound.
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;

#[test]
fn authored_index_checkpoint_decoding_does_not_mint_live_verification() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (runtime, created, equal) = indexed_equal_write(&execution);
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

fn named(value: impl serde::Serialize) -> Vec<u8> {
    rmp_serde::to_vec(&value).unwrap()
}

/// Replace the one occurrence of `from` in `bytes` with `to`.
fn splice(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let at: Vec<usize> = (0..=bytes.len() - from.len())
        .filter(|&at| bytes[at..].starts_with(from))
        .collect();
    assert_eq!(at.len(), 1, "the packet occurs once in the envelope");
    [&bytes[..at[0]], to, &bytes[at[0] + from.len()..]].concat()
}

/// Public #384: a persisted bound round-trips and must name the delta's prefix;
/// a legacy packet decodes without a bound, re-encodes byte-identically and
/// certifies nothing.
#[test]
fn authored_index_coverage_bound_round_trips_and_legacy_stays_unprovenanced() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (runtime, created, equal) = indexed_equal_write(&execution);
    let envelope = runtime
        .history
        .canonical_envelope(equal.commit.commit_id)
        .unwrap();
    let delta = envelope.owner_index_definition_delta().next_index_id;
    let encoded = rmp_serde::to_vec_named(envelope.as_ref()).unwrap();
    let decoded: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&encoded).unwrap();
    let writes = decoded.owner_authored_index_writes();
    let (bound, entries) = writes.persisted_entries().unwrap();
    assert_eq!(
        bound,
        Some(delta),
        "the bound is the captured catalog prefix"
    );
    assert!(!entries.is_empty());
    assert!(!writes.is_legacy());
    assert_eq!(rmp_serde::to_vec_named(&decoded).unwrap(), encoded);

    let covered_head = [
        named("authored_index_writes"),
        vec![0x81],
        named("Covered"),
        vec![0x82],
        named("covered_next_index_id"),
        named(delta),
        named("entries"),
    ]
    .concat();
    let disagreeing = [
        &covered_head[..covered_head.len() - named(delta).len() - named("entries").len()],
        &named(delta + 1),
        &named("entries"),
    ]
    .concat();
    let mismatch = splice(&encoded, &covered_head, &disagreeing);
    let refused = rmp_serde::from_slice::<crate::history::data::CanonicalCommitEnvelope>(&mismatch)
        .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("bound disagrees with the catalog delta"),
        "{refused}"
    );

    let legacy_head = [
        named("authored_index_writes"),
        vec![0x81],
        named("Complete"),
    ]
    .concat();
    let legacy_bytes = splice(&encoded, &covered_head, &legacy_head);
    let legacy: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&legacy_bytes).unwrap();
    let legacy_writes = legacy.owner_authored_index_writes();
    assert!(legacy_writes.is_legacy());
    assert_eq!(legacy_writes.persisted_entries(), Some((None, entries)));
    assert_eq!(rmp_serde::to_vec_named(&legacy).unwrap(), legacy_bytes);

    let catalog = runtime.indexes.catalog_binding().capture_catalog();
    let certify = |writes: &CanonicalAuthoredIndexWrites| {
        execution
            .run_single_stage(|context| catalog.certify_readmitted_writes(writes, delta, context))
            .unwrap()
            .0
            .unwrap()
    };
    assert_eq!(certify(legacy_writes), None, "legacy carries no provenance");
    assert_eq!(
        certify(writes),
        Some(catalog.catalog.coverage_before(delta)),
        "the receiving catalog certifies the bound"
    );
    for outcome in [&created, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
