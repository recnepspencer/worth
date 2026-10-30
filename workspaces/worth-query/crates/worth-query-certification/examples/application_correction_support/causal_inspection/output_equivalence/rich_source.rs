//! Source-only materialization owns original canonical effects without a completed query.
use super::super::super::authentication::*;
use super::super::{distinct_subjects, journey as source_journey, SourceDisclosure};
use super::{journey::install, retry::head};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*, runtime::*,
};
fn detail() -> WorthQueryApplicationMaterializationRequest {
    WorthQueryApplicationMaterializationRequest::family(
        WorthQueryApplicationEvidenceFamily::QueryInspection,
    )
    .with_family(WorthQueryApplicationEvidenceFamily::SignalEvaluation)
    .with_source_effects()
    .with_conditional_history()
    .with_producer_evidence()
}
fn assert_original(report: &WorthQueryMaterializedPerformedSourceInspection) {
    use WorthQueryMaterializedFamilyOutcome as Outcome;
    use WorthQueryMaterializedSourceFieldEffect as Effect;
    let body = report.materialization();
    assert_eq!(
        body.conditional_history_outcome(),
        WorthQueryConditionalHistoryOutcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert!(body.conditional_history().is_none());
    assert_eq!(
        body.family_outcome(WorthQueryApplicationEvidenceFamily::QueryInspection),
        Outcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert_eq!(
        body.family_outcome(WorthQueryApplicationEvidenceFamily::SignalEvaluation),
        Outcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert!(body.original_query().is_none());
    assert_eq!(
        body.producer_outcome(),
        Outcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert!(body.producer_connection().is_none());
    assert!(body.producer_evaluation().is_none());
    assert_eq!(body.source_outcome(), Outcome::Provided);
    let effects = body.source_effects().unwrap();
    assert_eq!(effects.source_commit(), report.source_commit());
    let fields: Vec<_> = effects.fields().collect();
    assert_eq!(fields.len(), 3);
    let input = fields
        .iter()
        .find(|field| field.output_name() == Some("input"))
        .unwrap();
    assert_eq!(input.effect(), Effect::Present);
    assert_eq!(
        input.value(),
        Some(&WorthQueryMaterializedFieldValue::UInt64(4))
    );
    for name in ["calculated", "operand"] {
        let field = fields
            .iter()
            .find(|field| field.output_name() == Some(name))
            .unwrap();
        assert_eq!(field.effect(), Effect::Unchanged);
        assert!(field.value().is_none());
    }
    assert_eq!(
        report.lower_evidence_posture(),
        CausalObservationMissingReferencePosture::MissingOptional
    );
    let references: Vec<_> = report.evidence_references().collect();
    assert!(!references.is_empty());
    assert!(references.iter().all(
        |reference| reference.owner() == "query" && !reference.reference_identity().is_empty()
    ));
}
pub fn source_only_rich_survives_receipt_drop_and_rechecks_diagnostics_cancellation_and_release() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source4, producer4, read) =
        source_journey::perform(&application, 4, "rich-source-only-four");
    drop(read);
    let expected = source4.commit_reference().clone();
    let (source5, producer5, read) =
        source_journey::perform(&application, 5, "rich-source-only-five");
    drop(read);
    drop((source5, producer5));
    let native_head = head(&application);
    let access = || distinct_subjects::access(&application, &principal, &scope, "numeric");
    let cancellation = WorthQueryCancellationSource::new();
    assert!(application
        .runtime()
        .cancel_after_next_inspection_materialization_for_test(cancellation.clone()));
    let reference = application
        .runtime()
        .request(&principal, &scope)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .reference_only()
        .execute_governed(access())
        .unwrap();
    assert!(
        !cancellation.token().is_cancelled(),
        "reference-only never assembles source fields"
    );
    let observation_identity = reference.source_observation_identity().to_owned();
    let reference_identity = reference.report_identity().to_owned();
    reference.close();
    let missing_metadata = application
        .runtime()
        .request(&principal, &scope)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .materialize(detail())
        .execute_with_diagnostic(access())
        .err()
        .expect("source diagnostic capability does not supply source inspection metadata");
    assert_eq!(
        missing_metadata.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    drop(missing_metadata);
    assert!(!cancellation.token().is_cancelled());
    // A genuine diagnostic capability's own request cancellation cannot be borrowed past.
    let diagnostic_cancel = WorthQueryCancellationSource::new();
    let diagnostic_scope = WorthQueryRequestScope::new(None, diagnostic_cancel.token());
    let diagnostic =
        distinct_subjects::access(&application, &principal, &diagnostic_scope, "numeric");
    diagnostic_cancel.cancel();
    let denial = application
        .runtime()
        .request(&principal, &scope)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .materialize(detail())
        .execute_governed(access(), diagnostic)
        .err()
        .expect("actual cancelled diagnostic request refuses before source value availability");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(denial);
    assert!(!cancellation.token().is_cancelled());
    let authority = ExecutionAuthority::serial();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(32 * 1024 * 1024),
                    Some(100_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let before = lease.charged_memory_bytes();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    let denial = application
        .runtime()
        .request(&principal, &cancelled)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .materialize(detail())
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(access(), access())
        .err()
        .expect("actual post-source-body cancellation refuses rich publication");
    assert!(cancellation.token().is_cancelled());
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(denial);
    assert_eq!(lease.charged_memory_bytes(), before);
    assert_eq!(head(&application), native_head);
    let result = application
        .runtime()
        .request(&principal, &scope)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .materialize(detail())
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(access(), access())
        .unwrap();
    assert_eq!(result.source_commit(), &expected);
    assert_eq!(result.source_observation_identity(), observation_identity);
    assert_ne!(result.report_identity(), reference_identity);
    assert_original(&result);
    assert!(lease.charged_memory_bytes() > before);
    let (metadata, diagnostic) = (access(), access());
    application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(distinct_subjects::grant_status::Withdraw {
            grant: "inspection-grant".to_owned(),
        })
        .without_source()
        .idempotency(&"rich-source-only-withdraw".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let denial = application
        .runtime()
        .request(&principal, &scope)
        .inspect_source::<SourceDisclosure>(&source4)
        .why_performed()
        .materialize(detail())
        .execute_governed(metadata, diagnostic)
        .err()
        .expect("new source-only rich request rechecks the withdrawn current grant");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denial.materialization_refusal().is_none());
    drop(denial);
    drop((source4, producer4));
    assert_eq!(result.source_commit(), &expected);
    assert_original(&result);
    assert_eq!(readers.observe().active(), baseline);
    result.close();
    assert_eq!(lease.charged_memory_bytes(), before);
    assert_eq!(readers.observe().active(), baseline);
}
