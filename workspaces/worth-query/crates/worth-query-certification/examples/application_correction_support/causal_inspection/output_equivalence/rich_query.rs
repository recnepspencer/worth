//! A fresh rich request reads the original projection after the business head changes.
use super::super::super::{authentication::*, query::full};
use super::super::{
    distinct_subjects::{self, observation, query},
    journey as source_journey,
};
use super::{demand, journey::install, retry::head};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*, runtime::*,
};

fn detail() -> WorthQueryApplicationMaterializationRequest {
    WorthQueryApplicationMaterializationRequest::family(
        WorthQueryApplicationEvidenceFamily::QueryInspection,
    )
    .with_family(WorthQueryApplicationEvidenceFamily::BridgeRoute)
}
pub(super) fn assert_original(report: &WorthQueryMaterializedApplicationInspection<'_>) {
    use WorthQueryApplicationEvidenceFamily as Family;
    use WorthQueryMaterializedFamilyOutcome as Outcome;
    let body = report.materialization();
    assert_eq!(
        body.family_outcome(Family::QueryInspection),
        Outcome::Provided
    );
    assert_eq!(
        body.family_outcome(Family::BridgeRoute),
        Outcome::RequestedButNotProvidedBySelectedObservationMode
    );
    assert_eq!(
        body.family_outcome(Family::SignalEvaluation),
        Outcome::NotRequested
    );
    let projection = body
        .original_query()
        .expect("retained original is provided");
    let roots: Vec<_> = projection.rows().collect();
    assert_eq!(roots.len(), 1);
    let fields: Vec<_> = roots[0].fields().collect();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].result_path(), "root/field[0]");
    assert_eq!(fields[0].output_name(), Some("key"));
    assert!(
        fields[0].value() == &WorthQueryMaterializedFieldValue::String("observer-decoy".into())
    );
    let relations: Vec<_> = roots[0].relations().collect();
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].output_name(), Some("references"));
    let children: Vec<_> = relations[0].rows().collect();
    assert_eq!(children.len(), 1);
    let fields: Vec<_> = children[0].fields().collect();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].output_name(), Some("child_key"));
    assert!(fields[0].value() == &WorthQueryMaterializedFieldValue::String("observer".into()));
    assert_eq!(fields[1].output_name(), Some("calculated"));
    assert_eq!(
        fields[1].value(),
        &WorthQueryMaterializedFieldValue::UInt64(0)
    );
    let references: Vec<_> = report.evidence_references().collect();
    assert!(!references.is_empty());
    assert!(references
        .iter()
        .all(|reference| reference.owner() == "query"));
    assert!(references
        .iter()
        .all(|reference| !reference.reference_identity().is_empty()
            && !reference.binding_identity().is_empty()));
}
pub fn rich_original_query_survives_head_change_and_private_assembly_cancellation() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (_, _, source_read) = source_journey::perform(&application, 4, "rich-original-four");
    drop(source_read);
    let (_, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 1);
    let (original, row) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    assert_eq!(row.references, vec![("observer".to_owned(), 0)]);
    drop(read);
    let (_, _, source_read) = source_journey::perform(&application, 5, "rich-head-five");
    drop(source_read);
    let (_, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 1);
    drop(read);
    let current = application
        .runtime()
        .request(&principal, &scope)
        .query(full::Read {
            key: "observer".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(current.rows()[0].calculated, 1);
    drop(current);
    let native_head = head(&application);
    let access = || distinct_subjects::access(&application, &principal, &scope, "observer-decoy");
    let cancellation = WorthQueryCancellationSource::new();
    assert!(application
        .runtime()
        .cancel_after_next_inspection_materialization_for_test(cancellation.clone()));
    let reference = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&original)
        .why_observed()
        .reference_only()
        .execute_governed(access())
        .unwrap();
    assert!(
        !cancellation.token().is_cancelled(),
        "reference-only never assembles a rich projection"
    );
    let observation_identity = reference.observation_identity().to_owned();
    let reference_identity = reference.report_identity().to_owned();
    reference.close();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
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
    let denial = application
        .runtime()
        .request(&principal, &cancelled)
        .inspect::<query::Binding>(&original)
        .why_observed()
        .materialize(detail())
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(access(), access())
        .err()
        .expect("real mid-assembly cancellation refuses publication");
    assert!(cancellation.token().is_cancelled());
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    assert!(denial.materialization_refusal().is_none());
    drop(denial);
    assert_eq!(lease.charged_memory_bytes(), before);
    assert_eq!(head(&application), native_head);
    assert_eq!(readers.observe().active(), baseline);
    let rich = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&original)
        .why_observed()
        .materialize(detail())
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(access(), access())
        .unwrap();
    assert!(lease.charged_memory_bytes() > before);
    assert_eq!(rich.observation_identity(), observation_identity);
    assert_ne!(rich.report_identity(), reference_identity);
    assert_original(&rich);
    assert_eq!(
        rich.materialization().source_outcome(),
        WorthQueryMaterializedFamilyOutcome::NotRequested
    );
    assert_eq!(
        rich.materialization().producer_outcome(),
        WorthQueryMaterializedFamilyOutcome::NotRequested
    );
    assert_eq!(head(&application), native_head);
    assert_eq!(readers.observe().active(), baseline);
    let (metadata, diagnostic) = (access(), access());
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(distinct_subjects::grant_status::Withdraw {
            grant: "observer-decoy-inspection-grant".to_owned(),
        })
        .without_source()
        .idempotency(&"rich-withdraw-current-inspection".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let denied = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&original)
        .why_observed()
        .materialize(detail())
        .execute_governed(metadata, diagnostic)
        .err()
        .expect("issued capabilities do not survive actual grant withdrawal");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(
        denied.materialization_refusal().is_none(),
        "no projection availability is disclosed before fresh admission"
    );
    drop(denied);
    assert_original(&rich);
    rich.close();
    assert_eq!(lease.charged_memory_bytes(), before);
    original.close();
    assert_eq!(readers.observe().active(), baseline);
}
