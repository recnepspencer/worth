use crate::facade::diagnostics::DiagnosticCode;
use crate::publication::cdc::data::{
    NormalizedContinuationProof, SubscriberContinuationClassSet, SubscriberContractDeclaration,
};
use crate::publication::cdc::planning::assess_subscriber_continuity;
use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion,
    HistoricalInterpretationSensitivity, SchemaBoundaryFingerprint, SchemaBridgeDescriptor,
    SchemaBridgeabilityClassification, SchemaContinuationClassification,
    SchemaContinuationDescriptor, SchemaStratum,
};
use crate::tests::support::{create_entity_outcome, diagnostic_field, runtime_with_test_schema};

#[test]
fn unsupported_continuation_failure_counts_current_boundary_when_no_prior_proof_exists() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "a");
    let fingerprint = SchemaBoundaryFingerprint::new([7_u8; 32]);
    let envelopes = vec![envelope_with_continuation(
        outcome.envelope().clone(),
        outcome.patch_position(),
        fingerprint,
        SchemaContinuationClassification::ContinueWithContractUpgrade,
    )];
    let contract = SubscriberContractDeclaration {
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([]),
        ..SubscriberContractDeclaration::default()
    };

    let failure = assess_subscriber_continuity(
        &runtime,
        &envelopes,
        &contract,
        &NormalizedContinuationProof::default(),
        DescriptorSemanticsVersion::default(),
    )
    .unwrap_err();

    let rejection_entry = failure
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated)
        .unwrap();
    assert_eq!(
        diagnostic_field(rejection_entry, "normalized_boundary_count_at_failure"),
        &crate::diagnostics::data::RelationalDiagnosticValue::Unsigned(1)
    );
}

#[test]
fn unsupported_continuation_failure_deduplicates_boundary_already_present_in_prior_proof() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "a");
    let fingerprint = SchemaBoundaryFingerprint::new([9_u8; 32]);
    let envelopes = vec![envelope_with_continuation(
        outcome.envelope().clone(),
        outcome.patch_position(),
        fingerprint,
        SchemaContinuationClassification::ContinueWithContractUpgrade,
    )];
    let contract = SubscriberContractDeclaration {
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([]),
        ..SubscriberContractDeclaration::default()
    };
    let prior_proof =
        NormalizedContinuationProof::new(vec![fingerprint], DescriptorSemanticsVersion::default());

    let failure = assess_subscriber_continuity(
        &runtime,
        &envelopes,
        &contract,
        &prior_proof,
        DescriptorSemanticsVersion::default(),
    )
    .unwrap_err();

    let rejection_entry = failure
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated)
        .unwrap();
    assert_eq!(
        diagnostic_field(rejection_entry, "normalized_boundary_count_at_failure"),
        &crate::diagnostics::data::RelationalDiagnosticValue::Unsigned(1)
    );
}

fn envelope_with_continuation(
    mut envelope: crate::history::data::CanonicalCommitEnvelope,
    position: crate::publication::patch::data::PatchStreamPosition,
    fingerprint: SchemaBoundaryFingerprint,
    continuation: SchemaContinuationClassification,
) -> crate::history::data::PositionedCanonicalCommit {
    envelope.schema_continuation_descriptor = Some(SchemaContinuationDescriptor::new(
        fingerprint,
        SchemaBridgeDescriptor::new(
            fingerprint,
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            continuation,
            SchemaBridgeabilityClassification::SubscriberVisible,
            HistoricalInterpretationSensitivity::NotSensitive,
            vec![SchemaStratum::PublicationContract],
        ),
        1,
    ));
    crate::history::data::PositionedCanonicalCommit::for_test(
        position,
        std::sync::Arc::new(envelope),
    )
}
