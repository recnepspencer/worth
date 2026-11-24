#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::publication::cdc::execution::collect_crossed_boundaries;
use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion,
    HistoricalInterpretationSensitivity, SchemaBoundaryFingerprint, SchemaBridgeDescriptor,
    SchemaBridgeabilityClassification, SchemaContinuationClassification,
    SchemaContinuationDescriptor,
};
use crate::tests::support::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommitFixturePort as _;

#[test]
fn crossed_boundary_collection_deduplicates_without_losing_first_seen_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "a");
    let second = create_entity_outcome(execution, &runtime, "b");
    let third = create_entity_outcome(execution, &runtime, "c");

    let fingerprint_a = SchemaBoundaryFingerprint::new([1_u8; 32]);
    let fingerprint_b = SchemaBoundaryFingerprint::new([2_u8; 32]);

    let mut first_envelope = copy_original_for_test(first.envelope(), execution);
    first_envelope
        .body_mut_for_test()
        .schema_continuation_descriptor = Some(SchemaContinuationDescriptor::new(
        fingerprint_a,
        SchemaBridgeDescriptor::new(
            fingerprint_a,
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            SchemaContinuationClassification::ContinueWithVisibleBridge,
            SchemaBridgeabilityClassification::SubscriberVisible,
            HistoricalInterpretationSensitivity::NotSensitive,
            Vec::new(),
        ),
        1,
    ));

    let mut second_envelope = copy_original_for_test(second.envelope(), execution);
    second_envelope
        .body_mut_for_test()
        .schema_continuation_descriptor = first_envelope
        .body_mut_for_test()
        .schema_continuation_descriptor
        .clone();

    let mut third_envelope = copy_original_for_test(third.envelope(), execution);
    third_envelope
        .body_mut_for_test()
        .schema_continuation_descriptor = Some(SchemaContinuationDescriptor::new(
        fingerprint_b,
        SchemaBridgeDescriptor::new(
            fingerprint_b,
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            SchemaContinuationClassification::ContinueWithVisibleBridge,
            SchemaBridgeabilityClassification::SubscriberVisible,
            HistoricalInterpretationSensitivity::NotSensitive,
            Vec::new(),
        ),
        1,
    ));

    let crossed = collect_crossed_boundaries(&[
        crate::history::data::PositionedCanonicalCommit::positioned_commit_for_test(
            first.patch_position(),
            std::sync::Arc::new(first_envelope),
        ),
        crate::history::data::PositionedCanonicalCommit::positioned_commit_for_test(
            second.patch_position(),
            std::sync::Arc::new(second_envelope),
        ),
        crate::history::data::PositionedCanonicalCommit::positioned_commit_for_test(
            third.patch_position(),
            std::sync::Arc::new(third_envelope),
        ),
    ]);

    assert_eq!(crossed, vec![fingerprint_a, fingerprint_b]);
}
