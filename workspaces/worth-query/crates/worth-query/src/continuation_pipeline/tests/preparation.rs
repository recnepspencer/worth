use crate::application::{
    WorthQueryDeclarationSignalCompatibilityChecked, WorthQueryDeclarationSignalCompatibilityInput,
    WorthQueryDeclarationSignalCompatibilitySupportRow,
};
use crate::continuation_pipeline::{
    WorthQueryContinuationBasisPosture, WorthQueryContinuationRuntimeContract,
    WorthQueryContinuationTruthContext, WorthQueryContinuationWorkspaceContract,
    WorthQueryPreparedContinuationBasisKind, WorthQueryPreparedContinuationFamily,
    WorthQueryPreparedContinuationOutcome,
};
use crate::WorthQueryEvidenceScope;

use super::support::{
    admitted_handle, context_request, envelope, historical_truth_view_request,
    preview_session_request, runtime_route_request, target_request, HistoricalFamily,
    PreviewFamily, RuntimeFamily,
};

#[test]
fn runtime_family_signal_compatibility_is_compatible_before_preparation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let support = handle.signal_compatibility_support::<super::support::Input<RuntimeFamily>>();
    let support_summary = support
        .rows()
        .iter()
        .map(|row: &WorthQueryDeclarationSignalCompatibilitySupportRow| {
            format!(
                "{}:{}:{}",
                row.execution_family().as_str(),
                row.basis_family().as_str(),
                row.status().as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    match handle.signal_compatibility_checked(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope::<RuntimeFamily>(
            &handle, "face-a",
        )),
    ) {
        WorthQueryDeclarationSignalCompatibilityChecked::Compatible(_) => {}
        WorthQueryDeclarationSignalCompatibilityChecked::Deferred(value) => panic!(
            "runtime continuation family unexpectedly deferred before preparation: {}; support rows = {}",
            value.reason(),
            support_summary
        ),
        WorthQueryDeclarationSignalCompatibilityChecked::Denied(value) => panic!(
            "runtime continuation family unexpectedly denied before preparation: {}; support rows = {}",
            value.reason(),
            support_summary
        ),
        WorthQueryDeclarationSignalCompatibilityChecked::Failed(value) => panic!(
            "runtime continuation family unexpectedly failed before preparation: {}; support rows = {}",
            value.reason(),
            support_summary
        ),
    }
}

#[test]
fn prepare_runtime_continuation_from_target_keeps_execution_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared continuation"),
    };
    assert_eq!(
        prepared.family(),
        WorthQueryPreparedContinuationFamily::BridgeRuntimeRoute
    );
    assert_eq!(
        prepared.truth_context(),
        WorthQueryContinuationTruthContext::Current
    );
    assert_eq!(
        prepared.runtime_contract(),
        WorthQueryContinuationRuntimeContract::RuntimeRoute
    );
    assert_eq!(
        prepared.workspace_contract(),
        WorthQueryContinuationWorkspaceContract::RuntimeWorkspace
    );
    assert!(!prepared.prepared_digest().is_empty());
}

#[test]
fn target_and_context_preparation_converge_for_equivalent_runtime_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let from_target =
        handle.prepare_continuation_from_target_checked(target_request::<RuntimeFamily>(
            &handle,
            "face-a",
            runtime_route_request(),
        ));
    let from_context =
        handle.prepare_continuation_from_context_checked(context_request(&handle, "face-a"));

    let target_prepared = match from_target.outcome() {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected target preparation"),
    };
    let context_prepared = match from_context.outcome() {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected context preparation"),
    };

    assert_eq!(
        target_prepared.prepared_digest(),
        context_prepared.prepared_digest()
    );
    assert_eq!(
        from_target.linked_artifacts(),
        from_context.linked_artifacts()
    );
}

#[test]
fn continuation_prepare_proof_exposes_target_specific_witnesses() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let proof = handle.prepare_continuation_from_target_proof(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    ));

    assert_eq!(proof.request().request_kind(), "prepared_continuation");
    assert!(matches!(
        proof.outcome(),
        WorthQueryPreparedContinuationOutcome::Prepared(_)
    ));
    assert_eq!(proof.witness_checks().len(), 3);
    assert_eq!(proof.witness_checks()[0].name(), "continuation_binding");
    assert!(proof.witness_checks()[0].did_pass());
    assert_eq!(proof.witness_checks()[1].name(), "signal_compatibility");
    assert_eq!(proof.witness_checks()[2].name(), "bridge_routing");
    assert!(proof.witness_checks()[2].did_pass());
    assert!(proof.linked_artifacts().declaration_digest().is_some());
    assert!(proof.linked_artifacts().route_plan_digest().is_some());
    assert!(proof.linked_artifacts().receipt_digest().is_some());
    assert!(proof.linked_artifacts().envelope_digest().is_some());
}

#[test]
fn preparation_keeps_current_historical_and_preview_truth_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");

    let current = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected current preparation"),
    };
    let historical = match handle.prepare_continuation_from_target(
        target_request::<HistoricalFamily>(&handle, "face-a", historical_truth_view_request()),
    ) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected historical preparation"),
    };
    let preview = match handle.prepare_continuation_from_target(target_request::<PreviewFamily>(
        &handle,
        "face-a",
        preview_session_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected preview preparation"),
    };

    assert_eq!(
        current.truth_context(),
        WorthQueryContinuationTruthContext::Current
    );
    assert_eq!(
        historical.truth_context(),
        WorthQueryContinuationTruthContext::Historical
    );
    assert_eq!(
        preview.truth_context(),
        WorthQueryContinuationTruthContext::Preview
    );
    assert_eq!(
        current.basis_posture(),
        WorthQueryContinuationBasisPosture::CurrentHead
    );
    assert_eq!(
        historical.basis_posture(),
        WorthQueryContinuationBasisPosture::HistoricalSnapshot
    );
    assert_eq!(
        preview.basis_posture(),
        WorthQueryContinuationBasisPosture::PreviewDerived
    );
    assert_eq!(
        current.execution_readmission().basis_witness().kind(),
        WorthQueryPreparedContinuationBasisKind::Current
    );
    assert_eq!(
        historical.execution_readmission().basis_witness().kind(),
        WorthQueryPreparedContinuationBasisKind::Historical
    );
    assert_eq!(
        preview.execution_readmission().basis_witness().kind(),
        WorthQueryPreparedContinuationBasisKind::PreviewDerived
    );
    assert_ne!(
        current
            .execution_readmission()
            .basis_witness()
            .basis_identity_digest(),
        historical
            .execution_readmission()
            .basis_witness()
            .basis_identity_digest()
    );
    assert_ne!(
        current
            .execution_readmission()
            .basis_witness()
            .basis_identity_digest(),
        preview
            .execution_readmission()
            .basis_witness()
            .basis_identity_digest()
    );
    let preview_witness = preview.execution_readmission().basis_witness();
    let preview_source_basis = preview_witness
        .source_basis_identity()
        .expect("preview continuation should retain source basis identity");
    let preview_lower_runtime_binding = preview_witness
        .expected_lower_runtime_binding_identity()
        .expect("preview continuation should retain lower-runtime binding identity");
    assert_eq!(
        preview_source_basis.scope(),
        WorthQueryEvidenceScope::ContinuationReadmissionSourceBasis
    );
    assert_eq!(
        preview_lower_runtime_binding.scope(),
        WorthQueryEvidenceScope::ContinuationReadmissionLowerRuntimeBinding
    );
    assert_ne!(
        preview_source_basis.as_str(),
        preview_lower_runtime_binding.as_str(),
        "source-basis identity and lower-runtime binding identity must stay typed as distinct roles"
    );
    assert_ne!(current.prepared_digest(), historical.prepared_digest());
    assert_ne!(current.prepared_digest(), preview.prepared_digest());
    assert_ne!(historical.prepared_digest(), preview.prepared_digest());
}
