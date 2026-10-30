//! Real owner eviction is distinct from immutable custody and fresh authorization.
use super::super::super::{authentication::*, host};
use super::super::{
    distinct_subjects::{self, observation, query},
    journey as source_journey, SourceDisclosure,
};
use super::{
    demand, journey::install, rich_connected::assert_source, rich_query::assert_original_projection,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*, runtime::*,
};
type Principal =
    WorthQueryAuthenticatedExternalPrincipal<super::super::super::schema::NumericSchema>;
fn metadata<'a>(
    application: &host::OutputHost,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    original: &'a WorthQueryApplicationQueryObservation,
) -> WorthQueryObservedApplicationInspection<'a> {
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(original)
        .why_observed()
        .reference_only()
        .execute_governed(distinct_subjects::access(
            application,
            principal,
            scope,
            "observer-decoy",
        ))
        .unwrap()
}
fn rich<'a>(
    application: &host::OutputHost,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    original: &'a WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
    detail: WorthQueryApplicationMaterializationRequest,
    allocation: ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryMaterializedApplicationInspection<'a>, WorthQueryApplicationInspectionDenial>
{
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(original)
        .why_observed()
        .evaluated_by::<SourceDisclosure>(source)
        .materialize(detail)
        .with_allocation_policy(allocation)
        .execute_governed(
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "numeric"),
            distinct_subjects::access(application, principal, scope, "numeric"),
        )
}
fn setup(
    application: &host::OutputHost,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
) -> (
    WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationQueryObservation,
) {
    // Select the actual bounded policy before the owner's first capture.
    // The three slots cover this supplier's readiness and its two output comparisons.
    assert!(application
        .runtime()
        .configure_conditional_history_for_test(true, 3));
    let (_, _, read) = source_journey::perform(application, 2, "history-source-two");
    drop(read);
    let (_, read, contacts) = demand::settle(application);
    assert_eq!(contacts, 1);
    drop(read);
    let (source, _, read) = source_journey::perform(application, 4, "history-source-four");
    drop(read);
    let (_, read, contacts) = demand::settle(application);
    assert_eq!(contacts, 1);
    let (original, row) = observation::read(application, principal, scope, &read, "observer-decoy");
    assert_eq!(row.references, vec![("observer".to_owned(), 0)]);
    drop(read);
    (source, original)
}
pub fn real_history_eviction_preserves_original_projection_source_and_evaluation() {
    use WorthQueryConditionalHistoryAvailability as Availability;
    use WorthQueryConditionalHistoryOutcome as Outcome;
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source4, original4) = setup(&application, &principal, &scope);
    let original_sibling = original4.clone();
    let available = metadata(&application, &principal, &scope, &original_sibling);
    assert_eq!(
        available.evidence_availability().conditional_history(),
        Some(Availability::RetainedAvailable)
    );
    let frozen = available.evidence_availability();
    let detail = WorthQueryApplicationMaterializationRequest::conditional_history()
        .with_family(WorthQueryApplicationEvidenceFamily::QueryInspection)
        .with_source_effects()
        .with_producer_evidence();
    let old_rich = rich(
        &application,
        &principal,
        &scope,
        &original4,
        &source4,
        detail,
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    assert_eq!(
        old_rich.materialization().conditional_history_outcome(),
        Outcome::Provided
    );
    let history = old_rich.materialization().conditional_history().unwrap();
    let original_history = (
        history.attempt(),
        history.decision_class(),
        history.output_version_before(),
        history.output_version(),
    );
    assert_eq!(
        old_rich
            .materialization()
            .producer_evaluation()
            .unwrap()
            .decision(),
        WorthQueryProducerEvaluationDecision::Equivalent
    );
    assert_source(&old_rich, 4, &source4);
    assert_original_projection(&old_rich);
    let old_identity = old_rich.report_identity().to_owned();
    let (source5, _, read) = source_journey::perform(&application, 5, "history-source-five");
    drop(read);
    let (_, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 1);
    let (original5, row5) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    assert_eq!(row5.references, vec![("observer".to_owned(), 1)]);
    drop(read);
    let changed =
        super::journey::inspect(&application, &principal, &scope, &original5, &source5).unwrap();
    assert_eq!(
        changed.producer_evaluation().decision(),
        WorthQueryProducerEvaluationDecision::Changed
    );
    assert_eq!(
        changed.producer_evaluation().source_commit(),
        source5.commit_reference()
    );
    changed.close();
    assert_eq!(
        available.evidence_availability(),
        frozen,
        "published metadata does not perform another owner lookup"
    );
    let lost = metadata(&application, &principal, &scope, &original_sibling);
    assert_eq!(
        lost.evidence_availability().conditional_history(),
        Some(Availability::EvictedByRetention)
    );
    assert_eq!(
        lost.evidence_availability().projection(),
        WorthQueryInspectionRetainedEvidence::Retained
    );
    assert_eq!(
        lost.evidence_availability().native_observation(),
        WorthQueryInspectionRetainedEvidence::Retained
    );
    lost.close();
    let newer = rich(
        &application,
        &principal,
        &scope,
        &original_sibling,
        &source4,
        detail,
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    assert_eq!(
        newer.materialization().conditional_history_outcome(),
        Outcome::OwnerUnavailable(Availability::EvictedByRetention)
    );
    assert!(newer.materialization().conditional_history().is_none());
    assert_source(&newer, 4, &source4);
    assert_original_projection(&newer);
    assert_eq!(
        newer
            .materialization()
            .producer_evaluation()
            .unwrap()
            .decision(),
        WorthQueryProducerEvaluationDecision::Equivalent
    );
    assert_ne!(newer.report_identity(), old_identity);
    let still = old_rich.materialization().conditional_history().unwrap();
    assert_eq!(
        (
            still.attempt(),
            still.decision_class(),
            still.output_version_before(),
            still.output_version()
        ),
        original_history,
        "a previously materialized readonly snapshot survives later owner eviction"
    );
    newer.close();
    old_rich.close();
    available.close();
    original4.close();
    let survivor = metadata(&application, &principal, &scope, &original_sibling);
    assert_eq!(
        survivor.evidence_availability().conditional_history(),
        Some(Availability::EvictedByRetention)
    );
    survivor.close();
    drop((source4, source5));
    original_sibling.close();
    original5.close();
    assert_eq!(readers.observe().active(), baseline);
}
pub fn history_omission_diagnostics_and_actual_private_cancellation_remain_distinct() {
    use WorthQueryConditionalHistoryAvailability as Availability;
    use WorthQueryConditionalHistoryOutcome as Outcome;
    let application = install();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    assert!(application
        .runtime()
        .configure_conditional_history_for_test(false, 1));
    let (_, _, read) = source_journey::perform(&application, 2, "history-omitted-two");
    drop(read);
    let (_, read, _) = demand::settle(&application);
    let (omitted, _) = observation::read(&application, &principal, &scope, &read, "observer-decoy");
    drop(read);
    let report = metadata(&application, &principal, &scope, &omitted);
    assert_eq!(
        report.evidence_availability().conditional_history(),
        Some(Availability::OmittedByTier)
    );
    report.close();
    let plain = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&omitted)
        .why_observed()
        .materialize(WorthQueryApplicationMaterializationRequest::conditional_history())
        .execute_governed(
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
        )
        .unwrap();
    assert_eq!(
        plain.materialization().conditional_history_outcome(),
        Outcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert!(plain.materialization().conditional_history().is_none());
    plain.close();
    omitted.close();
    drop((principal, application));
    let application = install();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source, original) = setup(&application, &principal, &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
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
    let charged = lease.charged_memory_bytes();
    let cancellation = WorthQueryCancellationSource::new();
    assert!(application
        .runtime()
        .cancel_after_next_inspection_materialization_for_test(cancellation.clone()));
    let reference = metadata(&application, &principal, &scope, &original);
    assert!(
        !cancellation.token().is_cancelled(),
        "availability metadata does not materialize history"
    );
    reference.close();
    let denied = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&original)
        .why_observed()
        .evaluated_by::<SourceDisclosure>(&source)
        .materialize(WorthQueryApplicationMaterializationRequest::conditional_history())
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
        )
        .err()
        .expect("query diagnostics cannot lend source diagnostics");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    drop(denied);
    assert!(!cancellation.token().is_cancelled());
    assert_eq!(lease.charged_memory_bytes(), charged);
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    let denied = rich(
        &application,
        &principal,
        &cancelled,
        &original,
        &source,
        WorthQueryApplicationMaterializationRequest::conditional_history(),
        ExecutionAllocationPolicy::Execution(&lease),
    )
    .err()
    .expect("actual cancellation after history body prevents publication");
    assert!(cancellation.token().is_cancelled());
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(denied);
    assert_eq!(lease.charged_memory_bytes(), charged);
    let successful = rich(
        &application,
        &principal,
        &scope,
        &original,
        &source,
        WorthQueryApplicationMaterializationRequest::conditional_history(),
        ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap();
    assert_eq!(
        successful.materialization().conditional_history_outcome(),
        Outcome::Provided
    );
    assert!(lease.charged_memory_bytes() > charged);
    successful.close();
    assert_eq!(lease.charged_memory_bytes(), charged);
    assert_eq!(readers.observe().active(), baseline);
    drop(source);
    original.close();
}
