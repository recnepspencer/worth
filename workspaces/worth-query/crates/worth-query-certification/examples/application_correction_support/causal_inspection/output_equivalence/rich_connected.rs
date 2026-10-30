//! Actual canonical source values and qualified producer evidence stay on the original.
use super::super::super::{authentication::*, host};
use super::super::{
    distinct_subjects::{self, observation, query},
    journey as source_journey, SourceDisclosure,
};
use super::{
    demand,
    journey::{inspect, install},
    retry::head,
    rich_query::assert_original,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*, runtime::*,
};
fn detail() -> WorthQueryApplicationMaterializationRequest {
    WorthQueryApplicationMaterializationRequest::family(
        WorthQueryApplicationEvidenceFamily::QueryInspection,
    )
    .with_family(WorthQueryApplicationEvidenceFamily::BridgeRoute)
    .with_source_effects()
    .with_producer_evidence()
}
fn rich<'o>(
    application: &host::OutputHost,
    principal: &WorthQueryAuthenticatedExternalPrincipal<
        super::super::super::schema::NumericSchema,
    >,
    scope: &WorthQueryRequestScope,
    original: &'o WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
) -> Result<WorthQueryMaterializedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
{
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(original)
        .why_observed()
        .evaluated_by::<SourceDisclosure>(source)
        .materialize(detail())
        .execute_governed(
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "numeric"),
            distinct_subjects::access(application, principal, scope, "numeric"),
        )
}
pub(super) fn assert_source(
    report: &WorthQueryMaterializedApplicationInspection<'_>,
    expected: u64,
    source: &WorthQueryApplicationCommitReceipt,
) {
    use WorthQueryMaterializedSourceFieldEffect as Effect;
    let body = report.materialization();
    assert_eq!(
        body.source_outcome(),
        WorthQueryMaterializedFamilyOutcome::Provided
    );
    let effects = body.source_effects().unwrap();
    assert_eq!(effects.source_commit(), source.commit_reference());
    let fields: Vec<_> = effects.fields().collect();
    assert_eq!(fields.len(), 3);
    let input = fields
        .iter()
        .find(|field| field.output_name() == Some("input"))
        .unwrap();
    assert_eq!(input.effect(), Effect::Present);
    assert_eq!(
        input.value(),
        Some(&WorthQueryMaterializedFieldValue::UInt64(expected))
    );
    for name in ["calculated", "operand"] {
        let field = fields
            .iter()
            .find(|field| field.output_name() == Some(name))
            .unwrap();
        assert_eq!(
            field.effect(),
            Effect::Unchanged,
            "other producer publications are not part of the selected source patch"
        );
        assert!(
            field.value().is_none(),
            "an unchanged field cannot borrow a value from the current head"
        );
    }
}
pub fn rich_source_and_producer_keep_original_values_both_authorities_and_release() {
    for withdrawn in ["inspection-grant", "observer-decoy-inspection-grant"] {
        let application = install();
        let readers = application.application_query_basis_observer();
        let baseline = readers.observe().active();
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (source2, _, read) = source_journey::perform(&application, 2, "rich-connected-two");
        drop(read);
        let (_, read, contacts) = demand::settle(&application);
        assert_eq!(contacts, 1);
        drop(read);
        let (source4, _, read) = source_journey::perform(&application, 4, "rich-connected-four");
        drop(read);
        let (producer4, read, contacts) = demand::settle(&application);
        assert_eq!(contacts, 1);
        let (original4, row4) =
            observation::read(&application, &principal, &scope, &read, "observer-decoy");
        assert_eq!(row4.references, vec![("observer".to_owned(), 0)]);
        drop(read);
        let reference = inspect(&application, &principal, &scope, &original4, &source4).unwrap();
        assert_eq!(
            reference.producer_evaluation().decision(),
            WorthQueryProducerEvaluationDecision::Equivalent
        );
        let revision = reference.producer_evaluation().observed_output_revision();
        let (source5, _, read) = source_journey::perform(&application, 5, "rich-connected-five");
        drop(read);
        let (producer5, read, contacts) = demand::settle(&application);
        assert_eq!(contacts, 1);
        let (original5, row5) =
            observation::read(&application, &principal, &scope, &read, "observer-decoy");
        assert_eq!(row5.references, vec![("observer".to_owned(), 1)]);
        drop(read);
        let selected_head = head(&application);
        let wrong = rich(&application, &principal, &scope, &original4, &source2)
            .err()
            .expect("equal parity does not join an older source");
        assert_eq!(
            wrong.producer_connection_refusal(),
            Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
        );
        drop(wrong);
        let access = |key| distinct_subjects::access(&application, &principal, &scope, key);
        let creation = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&original4)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source4)
            .materialize(detail())
            .execute_governed(
                access("observer-decoy"),
                access("observer-decoy"),
                access("numeric"),
                access("numeric"),
            )
            .err()
            .expect("richness cannot turn Equivalent into output creation");
        assert_eq!(
            creation.producer_connection_refusal(),
            Some(WorthQueryProducerConnectionDenial::OutputUnconnected)
        );
        drop(creation);
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
        let cancellation = WorthQueryCancellationSource::new();
        assert!(application
            .runtime()
            .cancel_after_next_inspection_materialization_for_test(cancellation.clone()));
        let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
        let denied = application
            .runtime()
            .request(&principal, &cancelled)
            .inspect::<query::Binding>(&original4)
            .why_observed()
            .evaluated_by::<SourceDisclosure>(&source4)
            .materialize(detail())
            .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
            .execute_governed(
                access("observer-decoy"),
                access("observer-decoy"),
                access("numeric"),
                access("numeric"),
            )
            .err()
            .expect("actual cancellation after both private bodies prevents public artifact");
        assert!(cancellation.token().is_cancelled());
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationInspectionStopKind::Request
        );
        drop(denied);
        assert_eq!(lease.charged_memory_bytes(), before);
        assert_eq!(head(&application), selected_head);
        let result = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&original4)
            .why_observed()
            .evaluated_by::<SourceDisclosure>(&source4)
            .materialize(detail())
            .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
            .execute_governed(
                access("observer-decoy"),
                access("observer-decoy"),
                access("numeric"),
                access("numeric"),
            )
            .unwrap();
        assert!(lease.charged_memory_bytes() > before);
        assert_original(&result);
        assert_source(&result, 4, &source4);
        assert_eq!(
            result.materialization().producer_outcome(),
            WorthQueryMaterializedFamilyOutcome::Provided
        );
        assert!(result.materialization().producer_connection().is_none());
        let evaluation = result.materialization().producer_evaluation().unwrap();
        assert!(evaluation.same_evaluation_as(reference.producer_evaluation()));
        assert_eq!(evaluation.producer_commit(), producer4.commit_reference());
        assert_eq!(evaluation.observed_output_revision(), revision);
        assert!(result.connection_identity().is_some());
        assert_ne!(
            result.connection_identity().unwrap(),
            result.report_identity()
        );
        let created = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&original5)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source5)
            .materialize(detail())
            .execute_governed(
                access("observer-decoy"),
                access("observer-decoy"),
                access("numeric"),
                access("numeric"),
            )
            .unwrap();
        assert_source(&created, 5, &source5);
        let connection = created.materialization().producer_connection().unwrap();
        assert_eq!(connection.source_commit(), source5.commit_reference());
        assert_eq!(connection.producer_commit(), producer5.commit_reference());
        assert_eq!(
            connection.output_publication_commit(),
            producer5.commit_reference()
        );
        assert!(created.materialization().producer_evaluation().is_none());
        created.close();
        original5.close();
        let (qm, qd, sm, sd) = (
            access("observer-decoy"),
            access("observer-decoy"),
            access("numeric"),
            access("numeric"),
        );
        application
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(distinct_subjects::grant_status::Withdraw {
                grant: withdrawn.to_owned(),
            })
            .without_source()
            .idempotency(&format!("rich-connected-withdraw-{withdrawn}"))
            .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let denial = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&original4)
            .why_observed()
            .evaluated_by::<SourceDisclosure>(&source4)
            .materialize(detail())
            .execute_governed(qm, qd, sm, sd)
            .err()
            .expect("either independently governed subject can revoke a new rich request");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationInspectionStopKind::Admission
        );
        assert!(denial.producer_evaluation_refusal().is_none());
        drop(denial);
        assert_original(&result);
        assert_source(&result, 4, &source4);
        reference.close();
        result.close();
        assert_eq!(lease.charged_memory_bytes(), before);
        original4.close();
        assert_eq!(readers.observe().active(), baseline);
    }
}
